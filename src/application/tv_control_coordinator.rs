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
use crate::domain::DeviceId;
use crate::domain::RemoteAction;

#[derive(Clone)]
pub struct AppServices {
    pub setup: Option<Arc<dyn TvSetupPort>>,
    pub discovery: Option<Arc<dyn TvDiscovery>>,
    pub gateway: Option<Arc<dyn TvGateway>>,
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
    ) -> Self {
        Self {
            setup,
            discovery: Some(discovery),
            gateway: Some(gateway),
        }
    }

    #[cfg(test)]
    pub fn without_adapters() -> Self {
        Self {
            setup: None,
            discovery: None,
            gateway: None,
        }
    }
}

pub struct ActiveRuntime {
    pub generation: u64,
    pub session_id: u64,
    pub control: Box<dyn TvSessionControl>,
}

type PendingConnection = Option<(SavedDevice, Box<dyn TvConnection>)>;

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
}

impl ConnectPlan {
    pub async fn run(self) -> Result<SessionPackage, ConnectFlowError> {
        TvControlCoordinator::connect_flow(
            self.service,
            self.gateway,
            self.id,
            self.pair_epoch,
            self.epoch,
            self.guard,
        )
        .await
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
        let list_error = match service.saved_devices() {
            Ok(devices) => {
                self.saved_devices = devices;
                false
            }
            Err(_) => true,
        };
        let selected = service.selected_device();
        if let Ok(Some(device)) = &selected {
            self.app_state.select_device(device.display());
            let generation = self.app_state.selection_generation();
            self.app_state
                .set_verified_actions(generation, device.verified_actions.clone());
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
        })
    }

    pub fn begin_forget(&mut self) -> Option<ForgetPlan> {
        if self.forget_pending.is_some() {
            return None;
        }
        let service = self.service.clone()?;
        let id = self.app_state.selected_device()?;
        self.stop_active();
        self.bump_pair_attempt();
        self.pair_pending = false;
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        self.app_state.clear_selection();
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
                self.app_state
                    .set_verified_actions(generation, device.verified_actions);
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
                let verified_actions = self
                    .saved_devices
                    .iter()
                    .find(|saved| saved.id == device.id)
                    .map(|saved| saved.verified_actions.clone())
                    .unwrap_or(device.verified_actions);
                self.app_state
                    .set_verified_actions(generation, verified_actions);
                ConnectCompletion::Connected { connection }
            }
            Err(error) => {
                self.app_state.set_connection_state(
                    generation,
                    super::control_state::ConnectionState::Failed,
                );
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
        let generation = self.app_state.selection_generation();
        self.app_state
            .set_verified_actions(generation, device.verified_actions.clone());
        Ok(device)
    }

    pub fn set_action_verified(
        &mut self,
        action: RemoteAction,
        verified: bool,
    ) -> Result<(), RepositoryError> {
        let id = self
            .app_state
            .selected_device()
            .ok_or(RepositoryError::Unavailable)?;
        let service = self.service.as_ref().ok_or(RepositoryError::Unavailable)?;
        let actions = service.set_action_verified(id, action, verified)?;
        if let Some(saved) = self.saved_devices.iter_mut().find(|saved| saved.id == id) {
            saved.verified_actions = actions.clone();
        }
        let generation = self.app_state.selection_generation();
        self.app_state.set_verified_actions(generation, actions);
        Ok(())
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
    use crate::domain::DeviceDisplay;
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
            verified_actions: vec![RemoteAction::Up, RemoteAction::Down],
        }
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
        coordinator
            .app_state
            .set_verified_actions(generation, device.verified_actions.clone());
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
}
