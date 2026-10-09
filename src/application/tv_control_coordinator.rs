use std::future::Future;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use futures_util::stream::BoxStream;

use super::certificate_trust::CertificatePin;
use super::control_state::ControlState;
use super::device_repository::{RepositoryError, SavedDevice};
use super::remote_dispatcher::{
    Admission, NotSentReason, RemoteDispatcher, RequestId, TerminalOutcome, TerminalResult,
};
use super::remote_request::SendRemoteAction;
use super::tv_address::{TargetError, TvHost};
use super::tv_discovery::{DiscoveryError, TvDiscovery};
use super::tv_session::{
    ProbeObservation, TvConnection, TvGateway, TvSessionControl, TvSessionError, TvSessionEvent,
};
use super::tv_setup_service::{ForgetResult, ReconnectError, SetupError, TvSetupPort};
use super::wake::WakeConfiguration;
use super::wake_transport::{WakePermit, WakeSendError, WakeTransport};
use crate::domain::DeviceId;

#[derive(Clone)]
pub struct AppServices {
    pub setup: Option<Arc<dyn TvSetupPort>>,
    pub discovery: Option<Arc<dyn TvDiscovery>>,
    pub gateway: Option<Arc<dyn TvGateway>>,
    pub wake: Option<Arc<dyn WakeTransport>>,
}

pub struct RestoreSummary {
    pub list_error: bool,
    pub selected: Result<Option<SavedDevice>, RepositoryError>,
}

impl AppServices {
    pub fn new(
        setup: Option<Arc<dyn TvSetupPort>>,
        discovery: Arc<dyn TvDiscovery>,
        gateway: Arc<dyn TvGateway>,
        wake: Arc<dyn WakeTransport>,
    ) -> Self {
        Self {
            setup,
            discovery: Some(discovery),
            gateway: Some(gateway),
            wake: Some(wake),
        }
    }

    #[cfg(test)]
    pub fn without_adapters() -> Self {
        Self {
            setup: None,
            discovery: None,
            gateway: None,
            wake: None,
        }
    }
}

pub struct ActiveRuntime {
    pub generation: u64,
    pub session_id: u64,
    pub control: Box<dyn TvSessionControl>,
}

type PendingConnection = Option<(SavedDevice, Box<dyn TvConnection>)>;

fn compact_saved_devices(
    service: &Arc<dyn TvSetupPort>,
    devices: Vec<SavedDevice>,
    selected: Option<DeviceId>,
) -> Vec<SavedDevice> {
    let mut compacted = Vec::new();

    for device in devices {
        let Some(existing_index) = compacted
            .iter()
            .position(|saved: &SavedDevice| saved.host == device.host)
        else {
            compacted.push(device);
            continue;
        };

        if compacted[existing_index].id == device.id {
            continue;
        }

        let keep_current = selected == Some(device.id);
        let duplicate_id = if keep_current {
            compacted[existing_index].id
        } else {
            device.id
        };
        if service.forget(duplicate_id).record_removed {
            if keep_current {
                compacted[existing_index] = device;
            }
        } else {
            compacted.push(device);
        }
    }

    compacted
}

pub struct SessionPackage(Arc<Mutex<PendingConnection>>);

impl SessionPackage {
    pub fn new(device: SavedDevice, connection: Box<dyn TvConnection>) -> Self {
        Self(Arc::new(Mutex::new(Some((device, connection)))))
    }

    pub fn take(&self) -> Option<(SavedDevice, Box<dyn TvConnection>)> {
        self.0.lock().ok()?.take()
    }
}

impl Clone for SessionPackage {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl std::fmt::Debug for SessionPackage {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SessionPackage([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairFlowError {
    Session(TvSessionError),
    Setup(SetupError),
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectFlowError {
    Session(TvSessionError),
    Reconnect(ReconnectError),
    CredentialSave,
    Cancelled,
}

pub struct DiscoveryPlan {
    pub attempt: u64,
    discovery: Arc<dyn TvDiscovery>,
}

impl DiscoveryPlan {
    pub async fn run(self) -> Result<Vec<TvHost>, DiscoveryError> {
        self.discovery.discover().await
    }
}

pub struct ProbePlan {
    pub attempt: u64,
    pub host: TvHost,
    gateway: Arc<dyn TvGateway>,
}

impl ProbePlan {
    pub async fn run(self) -> Result<ProbeObservation, TvSessionError> {
        self.gateway.probe(self.host).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairStartError {
    Busy,
    ProbeRequired,
    StorageUnavailable,
    TransportUnavailable,
}

pub struct PairPlan {
    pub attempt: u64,
    pub replace: Option<DeviceId>,
    service: Arc<dyn TvSetupPort>,
    gateway: Arc<dyn TvGateway>,
    host: TvHost,
    pin: CertificatePin,
    label: String,
    epoch: Arc<AtomicU64>,
    guard: Arc<Mutex<()>>,
}

impl PairPlan {
    pub async fn run(self) -> Result<SessionPackage, PairFlowError> {
        TvControlCoordinator::pair_flow(self).await
    }
}

pub struct ConnectPlan {
    pub attempt: u64,
    pub generation: u64,
    service: Arc<dyn TvSetupPort>,
    gateway: Arc<dyn TvGateway>,
    id: DeviceId,
    pair_epoch: u64,
    epoch: Arc<AtomicU64>,
    guard: Arc<Mutex<()>>,
    timeout: Option<std::time::Duration>,
}

impl ConnectPlan {
    pub async fn run(self) -> Result<SessionPackage, ConnectFlowError> {
        let flow = TvControlCoordinator::connect_flow(
            self.service,
            self.gateway,
            self.id,
            self.pair_epoch,
            self.epoch,
            self.guard,
        );
        if let Some(timeout) = self.timeout {
            tokio::time::timeout(timeout, flow)
                .await
                .map_err(|_| ConnectFlowError::Session(TvSessionError::Timeout))?
        } else {
            flow.await
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeStage {
    ConfigurationRequired,
    Idle,
    CheckingConnection,
    PacketSending,
    Reconnecting,
    Connected,
    Ready,
    Failed(WakeFailure),
    FailedAfterSend(WakeFailure),
    Cancelled,
    CancelledDuringSend,
    CancelledAfterSend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WakeFailure {
    Local(WakeSendError),
    Timeout,
    PairingRequired,
    Connection,
}

pub struct WakePlan {
    pub attempt: u64,
    pub generation: u64,
    pub device: DeviceId,
    host: TvHost,
    mac: crate::domain::MacAddress,
    transport: Arc<dyn WakeTransport>,
    permit: WakePermit,
}

impl WakePlan {
    pub async fn run(self) -> Result<(), WakeSendError> {
        if !self.permit.is_current() {
            return Err(WakeSendError::Cancelled);
        }
        self.transport
            .send_once(self.host, self.mac, self.permit)
            .await
    }
}

pub struct WakeReconnectPlan {
    pub attempt: u64,
    pub generation: u64,
    pub device: DeviceId,
    service: Arc<dyn TvSetupPort>,
    gateway: Arc<dyn TvGateway>,
    pair_epoch: u64,
    epoch: Arc<AtomicU64>,
    wake_epoch: Arc<AtomicU64>,
    guard: Arc<Mutex<()>>,
}

impl WakeReconnectPlan {
    pub async fn run(self) -> Result<SessionPackage, WakeFailure> {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            if self.wake_epoch.load(Ordering::SeqCst) != self.attempt {
                return Err(WakeFailure::Connection);
            }
            let flow = TvControlCoordinator::connect_flow(
                self.service.clone(),
                self.gateway.clone(),
                self.device,
                self.pair_epoch,
                self.epoch.clone(),
                self.guard.clone(),
            );
            match await_wake_work(self.attempt, &self.wake_epoch, deadline, flow).await? {
                Ok(package) => return Ok(package),
                Err(ConnectFlowError::Session(
                    TvSessionError::Offline
                    | TvSessionError::Timeout
                    | TvSessionError::RemoteChannel,
                )) => {}
                Err(error) => return Err(wake_connection_failure(error)),
            }
            if tokio::time::Instant::now() >= deadline {
                return Err(WakeFailure::Timeout);
            }
            await_wake_work(
                self.attempt,
                &self.wake_epoch,
                deadline,
                tokio::time::sleep_until(std::cmp::min(
                    deadline,
                    tokio::time::Instant::now() + std::time::Duration::from_millis(500),
                )),
            )
            .await?;
        }
    }
}

async fn await_wake_work<T>(
    attempt: u64,
    epoch: &AtomicU64,
    deadline: tokio::time::Instant,
    work: impl Future<Output = T>,
) -> Result<T, WakeFailure> {
    let mut cancellation_check = tokio::time::interval(std::time::Duration::from_millis(100));
    cancellation_check.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    tokio::pin!(work);
    loop {
        tokio::select! {
            biased;
            _ = cancellation_check.tick() => {
                if epoch.load(Ordering::SeqCst) != attempt {
                    return Err(WakeFailure::Connection);
                }
            }
            _ = tokio::time::sleep_until(deadline) => return Err(WakeFailure::Timeout),
            result = &mut work => return Ok(result),
        }
    }
}

pub struct ForgetPlan {
    pub id: DeviceId,
    service: Arc<dyn TvSetupPort>,
    guard: Arc<Mutex<()>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionImpact {
    Ignored,
    RemoteResult,
    Disconnected,
    TokenRejected,
}

pub enum PairCompletion {
    Ignored,
    StaleRepaired,
    StaleNew(ForgetPlan),
    Connected {
        generation: u64,
        connection: Box<dyn TvConnection>,
    },
    Failed(PairFlowError),
}

pub enum ConnectCompletion {
    Ignored,
    Connected { connection: Box<dyn TvConnection> },
    Failed(ConnectFlowError),
}

pub enum PowerStart {
    Toggle,
    Probe(ConnectPlan),
    MissingConfiguration,
    Busy,
    Stale,
    Unavailable,
}

pub enum PowerProbeCompletion {
    Ignored,
    Connected { connection: Box<dyn TvConnection> },
    Wake(WakePlan),
    Failed(WakeFailure),
}

fn wake_connection_failure(error: ConnectFlowError) -> WakeFailure {
    match error {
        ConnectFlowError::Session(
            TvSessionError::TokenRejected
            | TvSessionError::PairingDenied
            | TvSessionError::PairingTokenMissing,
        )
        | ConnectFlowError::Reconnect(
            ReconnectError::PairingRequired
            | ReconnectError::MissingTrust
            | ReconnectError::HostChanged
            | ReconnectError::Credential(super::credential_store::SecretError::InvalidToken)
            | ReconnectError::Trust(super::certificate_trust::TrustError::Corrupt),
        ) => WakeFailure::PairingRequired,
        _ => WakeFailure::Connection,
    }
}

impl ForgetPlan {
    pub async fn run(self) -> ForgetResult {
        TvControlCoordinator::forget_flow(self.service, self.id, self.guard).await
    }
}

pub struct TvControlCoordinator {
    pub app_state: ControlState,
    pub service: Option<Arc<dyn TvSetupPort>>,
    pub discovery: Option<Arc<dyn TvDiscovery>>,
    pub gateway: Option<Arc<dyn TvGateway>>,
    pub wake_transport: Option<Arc<dyn WakeTransport>>,
    pub wake_stage: WakeStage,
    pub wake_attempt: u64,
    pub wake_epoch: Arc<AtomicU64>,
    pub wake_send_guard: Arc<Mutex<()>>,
    pub saved_devices: Vec<SavedDevice>,
    pub dispatcher: RemoteDispatcher,
    pub active_runtime: Option<ActiveRuntime>,
    pub candidates: Vec<TvHost>,
    pub observed: Option<(TvHost, ProbeObservation)>,
    pub probe_attempt: u64,
    pub discovery_attempt: u64,
    pub pair_attempt: u64,
    pub pair_epoch: Arc<AtomicU64>,
    pub setup_guard: Arc<Mutex<()>>,
    pub pair_pending: bool,
    pub connect_attempt: u64,
    pub next_session_id: u64,
    pub forget_pending: Option<DeviceId>,
}

impl TvControlCoordinator {
    pub fn new(services: AppServices) -> Self {
        Self {
            app_state: ControlState::none(),
            service: services.setup,
            discovery: services.discovery,
            gateway: services.gateway,
            wake_transport: services.wake,
            wake_stage: WakeStage::ConfigurationRequired,
            wake_attempt: 0,
            wake_epoch: Arc::new(AtomicU64::new(0)),
            wake_send_guard: Arc::new(Mutex::new(())),
            saved_devices: Vec::new(),
            dispatcher: RemoteDispatcher::new(8, 64),
            active_runtime: None,
            candidates: Vec::new(),
            observed: None,
            probe_attempt: 0,
            discovery_attempt: 0,
            pair_attempt: 0,
            pair_epoch: Arc::new(AtomicU64::new(0)),
            setup_guard: Arc::new(Mutex::new(())),
            pair_pending: false,
            connect_attempt: 0,
            next_session_id: 0,
            forget_pending: None,
        }
    }

    pub fn restore(&mut self) -> Option<RestoreSummary> {
        let service = self.service.as_ref()?;
        let selected = service.selected_device();
        let selected_id = selected
            .as_ref()
            .ok()
            .and_then(|device| device.as_ref().map(|device| device.id));
        let list_error = match service.saved_devices() {
            Ok(devices) => {
                self.saved_devices = compact_saved_devices(service, devices, selected_id);
                false
            }
            Err(_) => true,
        };
        if let Ok(Some(device)) = &selected {
            if !self.saved_devices.iter().any(|saved| saved.id == device.id) {
                self.saved_devices.push(device.clone());
            }
            self.app_state.select_device(device.display());
            self.wake_stage = if device.wake.active_mac().is_some() {
                WakeStage::Idle
            } else {
                WakeStage::ConfigurationRequired
            };
        }
        Some(RestoreSummary {
            list_error,
            selected,
        })
    }

    pub fn bump_pair_attempt(&mut self) {
        self.pair_attempt = self.pair_attempt.saturating_add(1);
        self.pair_epoch.store(self.pair_attempt, Ordering::SeqCst);
    }

    pub fn address_changed(&mut self) {
        self.observed = None;
        self.probe_attempt = self.probe_attempt.saturating_add(1);
        self.bump_pair_attempt();
        self.pair_pending = false;
    }

    pub fn begin_discovery(&mut self) -> Option<DiscoveryPlan> {
        let discovery = self.discovery.clone()?;
        self.discovery_attempt = self.discovery_attempt.saturating_add(1);
        self.candidates.clear();
        Some(DiscoveryPlan {
            attempt: self.discovery_attempt,
            discovery,
        })
    }

    pub fn finish_discovery(
        &mut self,
        attempt: u64,
        result: &Result<Vec<TvHost>, DiscoveryError>,
    ) -> bool {
        if attempt != self.discovery_attempt {
            return false;
        }
        if let Ok(candidates) = result {
            self.candidates = candidates.clone();
        }
        true
    }

    pub fn begin_probe(&mut self, address: &str) -> Result<ProbePlan, TargetError> {
        let host = TvHost::parse(address)?;
        let Some(gateway) = self.gateway.clone() else {
            return Err(TargetError::InvalidHost);
        };
        self.probe_attempt = self.probe_attempt.saturating_add(1);
        self.bump_pair_attempt();
        self.pair_pending = false;
        self.observed = None;
        Ok(ProbePlan {
            attempt: self.probe_attempt,
            host,
            gateway,
        })
    }

    pub fn finish_probe(
        &mut self,
        attempt: u64,
        host: TvHost,
        result: &Result<ProbeObservation, TvSessionError>,
    ) -> bool {
        if attempt != self.probe_attempt {
            return false;
        }
        if let Ok(observation) = result {
            self.observed = Some((host, observation.clone()));
        }
        true
    }

    pub fn begin_pair(&mut self, replace: Option<DeviceId>) -> Result<PairPlan, PairStartError> {
        if self.pair_pending || self.forget_pending.is_some() {
            return Err(PairStartError::Busy);
        }
        let Some((host, observation)) = self.observed.clone() else {
            return Err(PairStartError::ProbeRequired);
        };
        let service = self
            .service
            .clone()
            .ok_or(PairStartError::StorageUnavailable)?;
        let gateway = self
            .gateway
            .clone()
            .ok_or(PairStartError::TransportUnavailable)?;
        let replace = replace.or_else(|| {
            self.saved_devices
                .iter()
                .find(|device| device.host == host)
                .map(|device| device.id)
        });
        self.cancel_wake();
        self.bump_pair_attempt();
        self.pair_pending = true;
        Ok(PairPlan {
            attempt: self.pair_attempt,
            replace,
            service,
            gateway,
            host,
            pin: observation.pin,
            label: observation.name.unwrap_or_else(|| "Samsung TV".to_owned()),
            epoch: Arc::clone(&self.pair_epoch),
            guard: Arc::clone(&self.setup_guard),
        })
    }

    pub fn begin_connect(&mut self) -> Option<ConnectPlan> {
        if self.forget_pending.is_some() {
            return None;
        }
        let service = self.service.clone()?;
        let gateway = self.gateway.clone()?;
        let id = self.app_state.selected_device()?;
        self.cancel_wake();
        self.wake_stage = if self.selected_wake_configured() {
            WakeStage::Idle
        } else {
            WakeStage::ConfigurationRequired
        };
        self.stop_active();
        self.bump_pair_attempt();
        self.pair_pending = false;
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        let generation = self.app_state.selection_generation();
        self.app_state.set_connection_state(
            generation,
            super::control_state::ConnectionState::Connecting,
        );
        Some(ConnectPlan {
            attempt: self.connect_attempt,
            generation,
            service,
            gateway,
            id,
            pair_epoch: self.pair_attempt,
            epoch: Arc::clone(&self.pair_epoch),
            guard: Arc::clone(&self.setup_guard),
            timeout: None,
        })
    }

    pub fn begin_power_probe(&mut self) -> Option<ConnectPlan> {
        let mut plan = self.begin_connect()?;
        plan.timeout = Some(std::time::Duration::from_secs(2));
        self.wake_stage = WakeStage::CheckingConnection;
        Some(plan)
    }

    pub fn begin_power_toggle(&mut self, request: &SendRemoteAction) -> PowerStart {
        if request.action() != crate::domain::RemoteAction::PowerToggle
            || self.app_state.selected_device() != Some(request.target())
            || self.app_state.selection_generation() != request.selection_generation()
        {
            return PowerStart::Stale;
        }
        if matches!(
            self.wake_stage,
            WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
        ) {
            return PowerStart::Busy;
        }
        if self.app_state.connection_state() == super::control_state::ConnectionState::Ready {
            return PowerStart::Toggle;
        }
        if !self.selected_wake_configured() {
            return PowerStart::MissingConfiguration;
        }
        self.begin_power_probe()
            .map(PowerStart::Probe)
            .unwrap_or(PowerStart::Unavailable)
    }

    pub fn finish_power_probe(
        &mut self,
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    ) -> PowerProbeCompletion {
        if self.wake_stage != WakeStage::CheckingConnection {
            return PowerProbeCompletion::Ignored;
        }
        match self.finish_connect(attempt, generation, result) {
            ConnectCompletion::Ignored => PowerProbeCompletion::Ignored,
            ConnectCompletion::Connected { connection } => {
                PowerProbeCompletion::Connected { connection }
            }
            ConnectCompletion::Failed(ConnectFlowError::Session(
                TvSessionError::Offline | TvSessionError::Timeout,
            )) => {
                self.wake_stage = WakeStage::Idle;
                self.begin_wake()
                    .map(PowerProbeCompletion::Wake)
                    .unwrap_or(PowerProbeCompletion::Failed(WakeFailure::Connection))
            }
            ConnectCompletion::Failed(error) => {
                let failure = wake_connection_failure(error);
                self.wake_stage = WakeStage::Failed(failure);
                PowerProbeCompletion::Failed(failure)
            }
        }
    }

    pub fn selected_wake_configured(&self) -> bool {
        self.app_state
            .selected_device()
            .and_then(|id| self.saved_devices.iter().find(|device| device.id == id))
            .is_some_and(|device| device.wake.active_mac().is_some())
    }

    pub fn cancel_wake(&mut self) {
        let send_guard = self.wake_send_guard.clone();
        let Ok(_send_guard) = send_guard.lock() else {
            return;
        };
        let checking = self.wake_stage == WakeStage::CheckingConnection;
        let active = matches!(
            self.wake_stage,
            WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
        );
        self.wake_attempt = self.wake_attempt.saturating_add(1);
        self.wake_epoch.store(self.wake_attempt, Ordering::SeqCst);
        if checking {
            self.connect_attempt = self.connect_attempt.saturating_add(1);
        }
        if active {
            self.bump_pair_attempt();
        }
        self.wake_stage = match self.wake_stage {
            WakeStage::CheckingConnection => WakeStage::Cancelled,
            WakeStage::PacketSending => WakeStage::CancelledDuringSend,
            WakeStage::Reconnecting => WakeStage::CancelledAfterSend,
            stage => stage,
        };
    }

    pub fn save_wake_configuration(
        &mut self,
        wake: WakeConfiguration,
    ) -> Result<SavedDevice, RepositoryError> {
        if wake.active.is_some() && wake.active_mac().is_none() {
            return Err(RepositoryError::Corrupt);
        }
        let id = self
            .app_state
            .selected_device()
            .ok_or(RepositoryError::Corrupt)?;
        let service = self.service.clone().ok_or(RepositoryError::Unavailable)?;
        self.cancel_wake();
        let device = {
            let _guard = self
                .setup_guard
                .lock()
                .map_err(|_| RepositoryError::Unavailable)?;
            service.save_wake_configuration(id, wake)?
        };
        if let Some(saved) = self.saved_devices.iter_mut().find(|saved| saved.id == id) {
            *saved = device.clone();
        } else {
            self.saved_devices.push(device.clone());
        }
        self.wake_stage =
            if self.app_state.connection_state() == super::control_state::ConnectionState::Ready {
                WakeStage::Connected
            } else if wake.active_mac().is_some() {
                WakeStage::Idle
            } else {
                WakeStage::ConfigurationRequired
            };
        Ok(device)
    }

    pub fn begin_wake(&mut self) -> Option<WakePlan> {
        if self.pair_pending
            || self.forget_pending.is_some()
            || matches!(
                self.wake_stage,
                WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
            )
            || self.app_state.connection_state() == super::control_state::ConnectionState::Ready
        {
            return None;
        }
        let device = self.app_state.selected_device()?;
        let saved = self.saved_devices.iter().find(|item| item.id == device)?;
        let mac = saved.wake.active_mac()?;
        let host = saved.host.clone();
        let transport = self.wake_transport.clone()?;
        if self.service.is_none() || self.gateway.is_none() {
            return None;
        }
        self.cancel_wake();
        self.bump_pair_attempt();
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        self.wake_stage = WakeStage::PacketSending;
        Some(WakePlan {
            attempt: self.wake_attempt,
            generation: self.app_state.selection_generation(),
            device,
            host,
            mac,
            transport,
            permit: WakePermit {
                epoch: self.wake_epoch.clone(),
                attempt: self.wake_attempt,
                send_guard: self.wake_send_guard.clone(),
            },
        })
    }

    fn wake_matches(&self, attempt: u64, generation: u64, device: DeviceId) -> bool {
        self.wake_attempt == attempt
            && self.app_state.selection_generation() == generation
            && self.app_state.selected_device() == Some(device)
    }

    pub fn finish_wake_send(
        &mut self,
        attempt: u64,
        generation: u64,
        device: DeviceId,
        result: Result<(), WakeSendError>,
    ) -> Option<WakeReconnectPlan> {
        if !self.wake_matches(attempt, generation, device)
            || self.wake_stage != WakeStage::PacketSending
        {
            return None;
        }
        if let Err(error) = result {
            self.wake_stage = WakeStage::Failed(WakeFailure::Local(error));
            return None;
        }
        self.wake_stage = WakeStage::Reconnecting;
        Some(WakeReconnectPlan {
            attempt,
            generation,
            device,
            service: self.service.clone()?,
            gateway: self.gateway.clone()?,
            pair_epoch: self.pair_attempt,
            epoch: self.pair_epoch.clone(),
            wake_epoch: self.wake_epoch.clone(),
            guard: self.setup_guard.clone(),
        })
    }

    pub fn finish_wake_reconnect(
        &mut self,
        attempt: u64,
        generation: u64,
        device: DeviceId,
        result: Result<SessionPackage, WakeFailure>,
    ) -> ConnectCompletion {
        if !self.wake_matches(attempt, generation, device)
            || self.wake_stage != WakeStage::Reconnecting
        {
            return ConnectCompletion::Ignored;
        }
        match result {
            Ok(package) => {
                let Some((saved, connection)) = package.take() else {
                    return ConnectCompletion::Ignored;
                };
                if saved.id != device {
                    return ConnectCompletion::Ignored;
                }
                self.app_state
                    .set_pairing_state(generation, super::control_state::PairingState::Ready);
                self.app_state
                    .set_connection_state(generation, super::control_state::ConnectionState::Ready);
                self.wake_stage = WakeStage::Ready;
                ConnectCompletion::Connected { connection }
            }
            Err(error) => {
                self.bump_pair_attempt();
                self.app_state.set_connection_state(
                    generation,
                    super::control_state::ConnectionState::Failed,
                );
                self.wake_stage = WakeStage::FailedAfterSend(error);
                ConnectCompletion::Failed(ConnectFlowError::Session(TvSessionError::Offline))
            }
        }
    }

    pub fn begin_forget(&mut self) -> Option<ForgetPlan> {
        if self.forget_pending.is_some() {
            return None;
        }
        let service = self.service.clone()?;
        let id = self.app_state.selected_device()?;
        self.cancel_wake();
        self.stop_active();
        self.bump_pair_attempt();
        self.pair_pending = false;
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        self.app_state.clear_selection();
        self.wake_stage = WakeStage::ConfigurationRequired;
        self.observed = None;
        self.forget_pending = Some(id);
        Some(ForgetPlan {
            id,
            service,
            guard: Arc::clone(&self.setup_guard),
        })
    }

    pub fn finish_forget(&mut self, id: DeviceId, result: ForgetResult) -> bool {
        if self.forget_pending != Some(id) {
            return false;
        }
        self.forget_pending = None;
        if result.record_removed {
            self.saved_devices.retain(|device| device.id != id);
        }
        true
    }

    pub fn finish_pair(
        &mut self,
        attempt: u64,
        replace: Option<DeviceId>,
        result: Result<SessionPackage, PairFlowError>,
    ) -> PairCompletion {
        if attempt != self.pair_attempt {
            if replace.is_some() {
                let Ok(package) = result else {
                    return PairCompletion::Ignored;
                };
                let Some((device, _)) = package.take() else {
                    return PairCompletion::Ignored;
                };
                if self.app_state.selected_device() == Some(device.id) {
                    if let Some(saved) = self
                        .saved_devices
                        .iter_mut()
                        .find(|saved| saved.id == device.id)
                    {
                        *saved = device.clone();
                    }
                    self.stop_active();
                    self.cancel_wake();
                    self.app_state.restart_selection(device.display());
                }
                return PairCompletion::StaleRepaired;
            }
            let (Ok(package), Some(service)) = (result, self.service.clone()) else {
                return PairCompletion::Ignored;
            };
            let Some((device, _)) = package.take() else {
                return PairCompletion::Ignored;
            };
            return PairCompletion::StaleNew(ForgetPlan {
                id: device.id,
                service,
                guard: Arc::clone(&self.setup_guard),
            });
        }
        self.pair_pending = false;
        match result {
            Ok(package) => {
                let Some((device, connection)) = package.take() else {
                    return PairCompletion::Ignored;
                };
                self.stop_active();
                self.cancel_wake();
                if self.app_state.selected_device() == Some(device.id) {
                    self.app_state.restart_selection(device.display());
                } else {
                    self.app_state.select_device(device.display());
                }
                if let Some(saved) = self
                    .saved_devices
                    .iter_mut()
                    .find(|saved| saved.id == device.id)
                {
                    *saved = device.clone();
                } else {
                    self.saved_devices.push(device.clone());
                }
                let generation = self.app_state.selection_generation();
                self.app_state
                    .set_pairing_state(generation, super::control_state::PairingState::Ready);
                self.app_state
                    .set_connection_state(generation, super::control_state::ConnectionState::Ready);
                self.wake_stage = WakeStage::Connected;
                PairCompletion::Connected {
                    generation,
                    connection,
                }
            }
            Err(error) => PairCompletion::Failed(error),
        }
    }

    pub fn finish_connect(
        &mut self,
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    ) -> ConnectCompletion {
        if attempt != self.connect_attempt || generation != self.app_state.selection_generation() {
            return ConnectCompletion::Ignored;
        }
        match result {
            Ok(package) => {
                let Some((device, connection)) = package.take() else {
                    return ConnectCompletion::Ignored;
                };
                if self.app_state.selected_device() != Some(device.id) {
                    return ConnectCompletion::Ignored;
                }
                self.app_state
                    .set_pairing_state(generation, super::control_state::PairingState::Ready);
                self.app_state
                    .set_connection_state(generation, super::control_state::ConnectionState::Ready);
                self.wake_stage = WakeStage::Connected;
                ConnectCompletion::Connected { connection }
            }
            Err(error) => {
                self.app_state.set_connection_state(
                    generation,
                    super::control_state::ConnectionState::Failed,
                );
                if self.wake_stage != WakeStage::CheckingConnection {
                    self.wake_stage = if self.selected_wake_configured() {
                        WakeStage::Idle
                    } else {
                        WakeStage::ConfigurationRequired
                    };
                }
                if matches!(
                    error,
                    ConnectFlowError::Session(TvSessionError::TokenRejected)
                        | ConnectFlowError::Reconnect(ReconnectError::PairingRequired)
                ) {
                    self.app_state
                        .set_pairing_state(generation, super::control_state::PairingState::Failed);
                }
                ConnectCompletion::Failed(error)
            }
        }
    }

    pub fn select_saved(&mut self, id: DeviceId) -> Result<SavedDevice, RepositoryError> {
        if self.forget_pending.is_some() {
            return Err(RepositoryError::Unavailable);
        }
        let service = self.service.clone().ok_or(RepositoryError::Unavailable)?;
        self.cancel_wake();
        self.bump_pair_attempt();
        let device = {
            let _guard = self
                .setup_guard
                .lock()
                .map_err(|_| RepositoryError::Unavailable)?;
            service.select_saved(id)?
        };
        self.stop_active();
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        if self.app_state.selected_device() == Some(device.id) {
            self.app_state.restart_selection(device.display());
        } else {
            self.app_state.select_device(device.display());
        }
        self.wake_stage = if device.wake.active_mac().is_some() {
            WakeStage::Idle
        } else {
            WakeStage::ConfigurationRequired
        };
        Ok(device)
    }

    pub fn stop_active(&mut self) {
        if let Some(mut active) = self.active_runtime.take() {
            active.control.abort();
        }
        self.dispatcher.abort_in_flight();
        self.dispatcher.cancel_waiting();
    }

    pub fn install_session(
        &mut self,
        generation: u64,
        connection: Box<dyn TvConnection>,
    ) -> (u64, BoxStream<'static, super::tv_session::TvSessionEvent>) {
        let active = connection.start();
        self.next_session_id = self.next_session_id.saturating_add(1);
        let session_id = self.next_session_id;
        // The caller schedules only the event stream. The session control
        // remains owned by the application coordinator.
        let super::tv_session::ActiveTvSession { control, events } = active;
        self.active_runtime = Some(ActiveRuntime {
            generation,
            session_id,
            control,
        });
        (session_id, events)
    }

    pub fn active_session_matches(&self, generation: u64, session_id: u64) -> bool {
        self.active_runtime.as_ref().is_some_and(|active| {
            active.generation == generation && active.session_id == session_id
        })
    }

    pub fn try_click(&self, id: RequestId, action: crate::domain::RemoteAction) -> bool {
        self.active_runtime
            .as_ref()
            .filter(|active| active.generation == self.app_state.selection_generation())
            .is_some_and(|active| active.control.try_click(id, action))
    }

    pub fn admit_remote(&mut self, request: SendRemoteAction) -> Admission {
        self.dispatcher.admit(&self.app_state, request)
    }

    pub fn pump_dispatch(&mut self) {
        let Some(item) = self.dispatcher.start_next(&self.app_state) else {
            return;
        };
        if !self.try_click(item.id, item.request.action()) {
            self.dispatcher.finish(
                item.id,
                TerminalOutcome::NotSent(NotSentReason::TransportUnavailable),
            );
        }
    }

    pub fn terminal_results_snapshot(&self) -> Vec<TerminalResult> {
        self.dispatcher.terminal_results().iter().cloned().collect()
    }

    pub fn acknowledge_terminal_through(&mut self, id: RequestId) {
        self.dispatcher.acknowledge_through(id);
    }

    pub fn process_session_event(
        &mut self,
        generation: u64,
        session_id: u64,
        event: TvSessionEvent,
    ) -> SessionImpact {
        if !self.active_session_matches(generation, session_id) {
            return SessionImpact::Ignored;
        }
        match event {
            TvSessionEvent::Written(id)
            | TvSessionEvent::Uncertain(id)
            | TvSessionEvent::NotSent(id) => {
                let outcome = match event {
                    TvSessionEvent::Written(_) => TerminalOutcome::Written,
                    TvSessionEvent::Uncertain(_) => TerminalOutcome::Uncertain,
                    _ => TerminalOutcome::NotSent(NotSentReason::TransportUnavailable),
                };
                if self.dispatcher.finish(id, outcome) {
                    SessionImpact::RemoteResult
                } else {
                    SessionImpact::Ignored
                }
            }
            TvSessionEvent::Disconnected | TvSessionEvent::TokenRejected => {
                self.active_runtime = None;
                self.dispatcher.abort_in_flight();
                self.dispatcher.cancel_waiting();
                self.app_state.set_connection_state(
                    generation,
                    super::control_state::ConnectionState::Failed,
                );
                self.wake_stage = if self.selected_wake_configured() {
                    WakeStage::Idle
                } else {
                    WakeStage::ConfigurationRequired
                };
                if matches!(event, TvSessionEvent::TokenRejected) {
                    self.app_state
                        .set_pairing_state(generation, super::control_state::PairingState::Failed);
                    SessionImpact::TokenRejected
                } else {
                    SessionImpact::Disconnected
                }
            }
        }
    }

    pub fn forget_result_when_unavailable() -> ForgetResult {
        ForgetResult {
            credential_removed: false,
            trust_removed: false,
            record_removed: false,
        }
    }

    async fn pair_flow(plan: PairPlan) -> Result<SessionPackage, PairFlowError> {
        let PairPlan {
            service,
            gateway,
            host,
            pin,
            label,
            replace,
            attempt,
            epoch,
            guard,
        } = plan;
        let mut connection = gateway
            .connect(host.clone(), pin, None)
            .await
            .map_err(PairFlowError::Session)?;
        let token = connection
            .await_authorized(false)
            .await
            .map_err(PairFlowError::Session)?
            .ok_or(PairFlowError::Session(TvSessionError::PairingTokenMissing))?;
        let device = tokio::task::spawn_blocking(move || {
            let _guard = guard.lock().map_err(|_| PairFlowError::Cancelled)?;
            if epoch.load(Ordering::SeqCst) != attempt {
                return Err(PairFlowError::Cancelled);
            }
            let device = match replace {
                Some(id) => service.repair(id, &label, host, pin, token),
                None => service.commit_pairing(&label, host, pin, token),
            }
            .map_err(PairFlowError::Setup)?;
            if replace.is_none() && epoch.load(Ordering::SeqCst) != attempt {
                return if service.forget(device.id).complete() {
                    Err(PairFlowError::Cancelled)
                } else {
                    Err(PairFlowError::Setup(SetupError::PartialCleanup))
                };
            }
            Ok(device)
        })
        .await
        .map_err(|_| PairFlowError::Setup(SetupError::PartialCleanup))??;
        Ok(SessionPackage::new(device, connection))
    }

    pub async fn connect_flow(
        service: Arc<dyn TvSetupPort>,
        gateway: Arc<dyn TvGateway>,
        id: DeviceId,
        pair_epoch: u64,
        epoch: Arc<AtomicU64>,
        guard: Arc<Mutex<()>>,
    ) -> Result<SessionPackage, ConnectFlowError> {
        let material = tokio::task::spawn_blocking({
            let service = service.clone();
            move || service.reconnect_material(id)
        })
        .await
        .map_err(|_| ConnectFlowError::Reconnect(ReconnectError::MissingDevice))?
        .map_err(ConnectFlowError::Reconnect)?;
        let mut connection = gateway
            .connect(
                material.device.host.clone(),
                material.pin,
                Some(material.token),
            )
            .await
            .map_err(ConnectFlowError::Session)?;
        if let Some(rotated) = connection
            .await_authorized(true)
            .await
            .map_err(ConnectFlowError::Session)?
        {
            tokio::task::spawn_blocking(move || {
                let _guard = guard.lock().map_err(|_| ConnectFlowError::Cancelled)?;
                if epoch.load(Ordering::SeqCst) != pair_epoch {
                    return Err(ConnectFlowError::Cancelled);
                }
                service
                    .save_rotated_token(id, &rotated)
                    .map_err(|_| ConnectFlowError::CredentialSave)
            })
            .await
            .map_err(|_| ConnectFlowError::CredentialSave)??;
        }
        Ok(SessionPackage::new(material.device, connection))
    }

    pub async fn forget_flow(
        service: Arc<dyn TvSetupPort>,
        id: DeviceId,
        guard: Arc<Mutex<()>>,
    ) -> ForgetResult {
        tokio::task::spawn_blocking(move || {
            let Ok(_guard) = guard.lock() else {
                return Self::forget_result_when_unavailable();
            };
            service.forget(id)
        })
        .await
        .unwrap_or_else(|_| Self::forget_result_when_unavailable())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::control_state::{ConnectionState, PairingState};
    use crate::domain::{DeviceDisplay, RemoteAction};
    use futures_util::stream;

    #[derive(Clone)]
    struct FakeControl {
        writes: Arc<Mutex<Vec<(RequestId, RemoteAction)>>>,
    }

    impl TvSessionControl for FakeControl {
        fn try_click(&self, id: RequestId, action: RemoteAction) -> bool {
            self.writes.lock().unwrap().push((id, action));
            true
        }

        fn abort(&mut self) {}
    }

    struct FakeConnection;

    impl TvConnection for FakeConnection {
        fn await_authorized(
            &mut self,
            _: bool,
        ) -> super::super::tv_session::SessionFuture<
            '_,
            Result<Option<super::super::credential_store::PairingToken>, TvSessionError>,
        > {
            Box::pin(async { Err(TvSessionError::Offline) })
        }

        fn start(self: Box<Self>) -> super::super::tv_session::ActiveTvSession {
            super::super::tv_session::ActiveTvSession {
                control: Box::new(FakeControl {
                    writes: Arc::new(Mutex::new(Vec::new())),
                }),
                events: Box::pin(stream::empty()),
            }
        }
    }

    fn saved_device(id: DeviceId) -> SavedDevice {
        SavedDevice {
            id,
            label: "TV".to_owned(),
            host: TvHost::parse("tv.local").unwrap(),
            wake: super::super::wake::WakeConfiguration::default(),
        }
    }

    #[derive(Clone)]
    struct DuplicateSetup {
        devices: Arc<Mutex<Vec<SavedDevice>>>,
    }

    impl TvSetupPort for DuplicateSetup {
        fn saved_devices(&self) -> Result<Vec<SavedDevice>, RepositoryError> {
            Ok(self.devices.lock().unwrap().clone())
        }

        fn selected_device(&self) -> Result<Option<SavedDevice>, RepositoryError> {
            Ok(None)
        }

        fn select_saved(&self, _: DeviceId) -> Result<SavedDevice, RepositoryError> {
            Err(RepositoryError::Corrupt)
        }

        fn commit_pairing(
            &self,
            _: &str,
            _: TvHost,
            _: CertificatePin,
            _: super::super::credential_store::PairingToken,
        ) -> Result<SavedDevice, SetupError> {
            Err(SetupError::Preferences(RepositoryError::Unavailable))
        }

        fn repair(
            &self,
            _: DeviceId,
            _: &str,
            _: TvHost,
            _: CertificatePin,
            _: super::super::credential_store::PairingToken,
        ) -> Result<SavedDevice, SetupError> {
            Err(SetupError::Preferences(RepositoryError::Unavailable))
        }

        fn reconnect_material(
            &self,
            _: DeviceId,
        ) -> Result<super::super::tv_setup_service::ReconnectMaterial, ReconnectError> {
            Err(ReconnectError::MissingDevice)
        }

        fn save_rotated_token(
            &self,
            _: DeviceId,
            _: &super::super::credential_store::PairingToken,
        ) -> Result<(), super::super::credential_store::SecretError> {
            Err(super::super::credential_store::SecretError::Unavailable)
        }

        fn save_wake_configuration(
            &self,
            _: DeviceId,
            _: WakeConfiguration,
        ) -> Result<SavedDevice, RepositoryError> {
            Err(RepositoryError::Unavailable)
        }

        fn forget(&self, id: DeviceId) -> ForgetResult {
            self.devices
                .lock()
                .unwrap()
                .retain(|device| device.id != id);
            ForgetResult {
                credential_removed: true,
                trust_removed: true,
                record_removed: true,
            }
        }
    }

    #[test]
    fn restore_compacts_same_host_records_and_keeps_the_selected_record() {
        let first = saved_device(DeviceId::new(1));
        let mut selected = saved_device(DeviceId::new(2));
        selected.label = "Main Screen".to_owned();
        let setup = DuplicateSetup {
            devices: Arc::new(Mutex::new(vec![first, selected.clone()])),
        };
        let service: Arc<dyn TvSetupPort> = Arc::new(setup.clone());

        let compacted =
            compact_saved_devices(&service, setup.saved_devices().unwrap(), Some(selected.id));

        assert_eq!(compacted, vec![selected.clone()]);
        assert_eq!(setup.saved_devices().unwrap(), vec![selected]);
    }

    fn connected_coordinator() -> (TvControlCoordinator, SavedDevice) {
        let mut coordinator = TvControlCoordinator::new(AppServices::without_adapters());
        let device = saved_device(DeviceId::new(7));
        coordinator
            .app_state
            .select_device(DeviceDisplay::new(device.id, device.label.clone()));
        let generation = coordinator.app_state.selection_generation();
        coordinator
            .app_state
            .set_pairing_state(generation, PairingState::Ready);
        coordinator
            .app_state
            .set_connection_state(generation, ConnectionState::Ready);
        (coordinator, device)
    }

    #[test]
    fn stale_session_result_cannot_finish_the_active_request() {
        let (mut coordinator, device) = connected_coordinator();
        let generation = coordinator.app_state.selection_generation();
        let writes = Arc::new(Mutex::new(Vec::new()));
        coordinator.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 2,
            control: Box::new(FakeControl {
                writes: Arc::clone(&writes),
            }),
        });
        let request = SendRemoteAction::new(device.id, generation, RemoteAction::Up);
        let Admission::Queued(id) = coordinator.admit_remote(request) else {
            panic!("request was not queued");
        };
        coordinator.pump_dispatch();
        assert_eq!(*writes.lock().unwrap(), vec![(id, RemoteAction::Up)]);
        assert_eq!(
            coordinator.process_session_event(generation, 1, TvSessionEvent::Written(id)),
            SessionImpact::Ignored
        );
        assert!(coordinator.terminal_results_snapshot().is_empty());
        assert_eq!(
            coordinator.process_session_event(generation, 2, TvSessionEvent::Written(id)),
            SessionImpact::RemoteResult
        );
        let results = coordinator.terminal_results_snapshot();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].outcome, TerminalOutcome::Written);
        coordinator.acknowledge_terminal_through(results[0].id);
        assert!(coordinator.terminal_results_snapshot().is_empty());
        assert_eq!(
            coordinator.process_session_event(generation, 2, TvSessionEvent::Written(id)),
            SessionImpact::Ignored
        );
    }

    #[test]
    fn token_rejection_marks_active_write_uncertain_and_cancels_waiting_work() {
        let (mut coordinator, device) = connected_coordinator();
        let generation = coordinator.app_state.selection_generation();
        coordinator.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 3,
            control: Box::new(FakeControl {
                writes: Arc::new(Mutex::new(Vec::new())),
            }),
        });
        for action in [RemoteAction::Up, RemoteAction::Down] {
            assert!(matches!(
                coordinator.admit_remote(SendRemoteAction::new(device.id, generation, action)),
                Admission::Queued(_)
            ));
        }
        coordinator.pump_dispatch();
        assert_eq!(
            coordinator.process_session_event(generation, 2, TvSessionEvent::TokenRejected),
            SessionImpact::Ignored
        );
        assert_eq!(
            coordinator.process_session_event(generation, 3, TvSessionEvent::TokenRejected),
            SessionImpact::TokenRejected
        );
        let results = coordinator.terminal_results_snapshot();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].outcome, TerminalOutcome::Uncertain);
        assert_eq!(
            results[1].outcome,
            TerminalOutcome::NotSent(NotSentReason::Cancelled)
        );
        assert_eq!(
            coordinator.app_state.connection_state(),
            ConnectionState::Failed
        );
        assert_eq!(coordinator.app_state.pairing_state(), PairingState::Failed);
    }

    #[test]
    fn stale_repair_restarts_the_selected_generation() {
        let (mut coordinator, device) = connected_coordinator();
        let prior_generation = coordinator.app_state.selection_generation();
        coordinator.saved_devices.push(device.clone());
        coordinator.pair_attempt = 2;
        let mut repaired = device.clone();
        repaired.label = "Repaired TV".to_owned();

        let completion = coordinator.finish_pair(
            1,
            Some(device.id),
            Ok(SessionPackage::new(
                repaired.clone(),
                Box::new(FakeConnection),
            )),
        );

        assert!(matches!(completion, PairCompletion::StaleRepaired));
        assert_eq!(coordinator.saved_devices, vec![repaired]);
        assert_eq!(
            coordinator.app_state.selection_generation(),
            prior_generation + 1
        );
        assert_eq!(
            coordinator.app_state.connection_state(),
            ConnectionState::NotConnected
        );
    }

    #[test]
    fn cancelling_wake_ignores_a_late_packet_completion() {
        let mut coordinator = TvControlCoordinator::new(AppServices::without_adapters());
        let device = saved_device(DeviceId::new(8));
        coordinator.app_state.select_device(device.display());
        let generation = coordinator.app_state.selection_generation();
        coordinator.wake_attempt = 4;
        coordinator.wake_epoch.store(4, Ordering::SeqCst);
        coordinator.wake_stage = WakeStage::PacketSending;
        coordinator.cancel_wake();
        assert_eq!(coordinator.wake_stage, WakeStage::CancelledDuringSend);
        assert_eq!(coordinator.wake_epoch.load(Ordering::SeqCst), 5);
        assert!(coordinator
            .finish_wake_send(4, generation, device.id, Ok(()))
            .is_none());
        assert_eq!(coordinator.wake_stage, WakeStage::CancelledDuringSend);
    }

    #[test]
    fn cancelling_reconnect_preserves_packet_sent_fact() {
        let mut coordinator = TvControlCoordinator::new(AppServices::without_adapters());
        coordinator.wake_stage = WakeStage::Reconnecting;

        coordinator.cancel_wake();

        assert_eq!(coordinator.wake_stage, WakeStage::CancelledAfterSend);
    }

    #[test]
    fn missing_or_invalid_pairing_material_stops_power_before_wake() {
        for error in [
            ConnectFlowError::Reconnect(ReconnectError::PairingRequired),
            ConnectFlowError::Reconnect(ReconnectError::MissingTrust),
            ConnectFlowError::Reconnect(ReconnectError::HostChanged),
            ConnectFlowError::Reconnect(ReconnectError::Credential(
                super::super::credential_store::SecretError::InvalidToken,
            )),
            ConnectFlowError::Session(TvSessionError::TokenRejected),
        ] {
            assert_eq!(wake_connection_failure(error), WakeFailure::PairingRequired);
        }
    }

    #[test]
    fn wake_becomes_ready_only_after_matching_reconnect_completion() {
        let mut coordinator = TvControlCoordinator::new(AppServices::without_adapters());
        let device = saved_device(DeviceId::new(15));
        coordinator.app_state.select_device(device.display());
        let generation = coordinator.app_state.selection_generation();
        coordinator.wake_attempt = 3;
        coordinator.wake_stage = WakeStage::Reconnecting;

        assert!(matches!(
            coordinator.finish_wake_reconnect(
                2,
                generation,
                device.id,
                Ok(SessionPackage::new(
                    device.clone(),
                    Box::new(FakeConnection)
                ))
            ),
            ConnectCompletion::Ignored
        ));
        assert_eq!(coordinator.wake_stage, WakeStage::Reconnecting);
        assert!(matches!(
            coordinator.finish_wake_reconnect(
                3,
                generation,
                device.id,
                Ok(SessionPackage::new(device, Box::new(FakeConnection)))
            ),
            ConnectCompletion::Connected { .. }
        ));
        assert_eq!(coordinator.wake_stage, WakeStage::Ready);
        assert_eq!(
            coordinator.app_state.connection_state(),
            ConnectionState::Ready
        );
    }

    #[test]
    fn invalid_wake_configuration_does_not_cancel_an_active_send() {
        let mut coordinator = TvControlCoordinator::new(AppServices::without_adapters());
        let device = saved_device(DeviceId::new(18));
        coordinator.app_state.select_device(device.display());
        coordinator.wake_stage = WakeStage::PacketSending;
        coordinator.wake_attempt = 4;

        assert!(matches!(
            coordinator.save_wake_configuration(WakeConfiguration {
                wired: None,
                wifi: None,
                active: Some(super::super::wake::WakeInterface::Wired),
            }),
            Err(RepositoryError::Corrupt)
        ));
        assert_eq!(coordinator.wake_stage, WakeStage::PacketSending);
        assert_eq!(coordinator.wake_attempt, 4);
    }

    #[tokio::test]
    async fn cancelling_wake_drops_an_in_flight_reconnect_wait() {
        let epoch = Arc::new(AtomicU64::new(4));
        let cancelled = epoch.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            cancelled.store(5, Ordering::SeqCst);
        });

        let result = await_wake_work(
            4,
            &epoch,
            tokio::time::Instant::now() + std::time::Duration::from_secs(1),
            std::future::pending::<()>(),
        )
        .await;

        assert_eq!(result, Err(WakeFailure::Connection));
    }

    #[tokio::test]
    async fn reconnect_wait_stops_at_its_deadline() {
        let epoch = AtomicU64::new(4);

        let result = await_wake_work(
            4,
            &epoch,
            tokio::time::Instant::now() + std::time::Duration::from_millis(10),
            std::future::pending::<()>(),
        )
        .await;

        assert_eq!(result, Err(WakeFailure::Timeout));
    }
}
