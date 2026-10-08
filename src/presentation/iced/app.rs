use super::message::{ConnectFlowError, Message, PairFlowError, SessionPackage, Shortcut};
use super::view;
use super::view_model::{rejection_message, remote_action_label, ViewModel};
use crate::application::device::SavedDevice;
use crate::application::device_service::DeviceService;
use crate::application::discovery::{DeviceDiscovery, DiscoveryError};
use crate::application::dispatcher::{
    Admission, DispatchRejection, Dispatcher, NotSentReason, TerminalOutcome,
};
use crate::application::target::TvHost;
use crate::application::SendRemoteAction;
use crate::application::{ConnectionState, PairingState};
use crate::infrastructure::keychain::KeychainSecretStore;
use crate::infrastructure::samsung::session::{self, SessionCommand, SessionEvent};
use crate::infrastructure::ssdp::SsdpDiscovery;
use crate::infrastructure::storage::{
    default_app_data_dir, LocalDeviceRepository, LocalTrustStore,
};
use crate::State;
use ::iced::keyboard::{self, key, Key, Modifiers};
use ::iced::{event, window, Element, Event, Size, Subscription, Task};
use futures_util::stream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

type LiveService = DeviceService<LocalDeviceRepository, KeychainSecretStore, LocalTrustStore>;

struct ActiveRuntime {
    generation: u64,
    session_id: u64,
    commands: mpsc::Sender<SessionCommand>,
    task: tokio::task::JoinHandle<()>,
}

pub struct App {
    main_window: Option<window::Id>,
    settings_window: Option<window::Id>,
    view_model: ViewModel,
    app_state: State,
    service: Option<Arc<LiveService>>,
    saved_devices: Vec<SavedDevice>,
    dispatcher: Dispatcher,
    active_runtime: Option<ActiveRuntime>,
    tv_address: String,
    candidates: Vec<TvHost>,
    observed: Option<(TvHost, session::ProbeObservation)>,
    settings_status: String,
    probe_attempt: u64,
    discovery_attempt: u64,
    pair_attempt: u64,
    pair_epoch: Arc<AtomicU64>,
    setup_guard: Arc<Mutex<()>>,
    pair_pending: bool,
    connect_attempt: u64,
    next_session_id: u64,
    forget_pending: Option<crate::DeviceId>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app_state = State::none();
        let view_model = ViewModel::new(&app_state);
        (
            Self {
                main_window: None,
                settings_window: None,
                view_model,
                app_state,
                service: None,
                saved_devices: Vec::new(),
                dispatcher: Dispatcher::new(8, 64),
                active_runtime: None,
                tv_address: String::new(),
                candidates: Vec::new(),
                observed: None,
                settings_status: "Enter a local TV address to begin secure setup.".to_owned(),
                probe_attempt: 0,
                discovery_attempt: 0,
                pair_attempt: 0,
                pair_epoch: Arc::new(AtomicU64::new(0)),
                setup_guard: Arc::new(Mutex::new(())),
                pair_pending: false,
                connect_attempt: 0,
                next_session_id: 0,
                forget_pending: None,
            },
            Task::done(Message::OpenMainWindow),
        )
    }

    fn boot() -> (Self, Task<Message>) {
        let (mut app, open) = Self::new();
        let Some(directory) = default_app_data_dir() else {
            app.settings_status = "Application Support is unavailable on this Mac.".to_owned();
            return (app, open);
        };
        let service = Arc::new(DeviceService::new(
            LocalDeviceRepository::new(&directory),
            KeychainSecretStore,
            LocalTrustStore::new(&directory),
        ));
        let mut tasks = vec![open];
        match service.saved_devices() {
            Ok(devices) => app.saved_devices = devices,
            Err(_) => app.settings_status = "Saved TV list needs attention.".to_owned(),
        }
        match service.selected_device() {
            Ok(Some(device)) => {
                app.tv_address = device.host.as_str().to_owned();
                app.app_state.select_device(device.display());
                app.app_state.set_verified_actions(
                    app.app_state.selection_generation(),
                    device.verified_actions,
                );
                app.view_model.update_control_state(&app.app_state);
                app.settings_status = "Restoring the selected TV securely.".to_owned();
                tasks.push(Task::done(Message::ConnectSelected));
            }
            Ok(None) => {}
            Err(_) => app.settings_status = "Saved TV settings need attention.".to_owned(),
        }
        app.service = Some(service);
        (app, Task::batch(tasks))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenMainWindow if self.main_window.is_none() => {
                let (main_window, open) = window::open(main_window_settings());
                self.main_window = Some(main_window);
                Task::batch([open.map(Message::MainWindowOpened)])
            }
            Message::OpenMainWindow => Task::none(),
            Message::Navigate(view) => {
                self.view_model.select_view(view);
                Task::none()
            }
            Message::OpenSettings if self.settings_window.is_none() => {
                let (settings_window, open) = window::open(settings_window_settings());
                self.settings_window = Some(settings_window);
                Task::batch([
                    open.map(Message::SettingsWindowOpened),
                    self.publish(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::MainWindow,
                        "Opened Settings.",
                    ),
                ])
            }
            Message::OpenSettings => Task::none(),
            Message::MainWindowOpened(_) => Task::none(),
            Message::SettingsWindowOpened(_) => Task::none(),
            Message::WindowClosed(id) if self.main_window == Some(id) => ::iced::exit(),
            Message::WindowClosed(id) if self.settings_window == Some(id) => {
                self.settings_window = None;
                self.publish(
                    super::view_model::MessageSeverity::Information,
                    super::view_model::MessageSource::SettingsWindow,
                    "Closed Settings.",
                )
            }
            Message::WindowClosed(_) => Task::none(),
            Message::ResizeMessages(height) => {
                self.view_model.resize_message_pane(height);
                Task::none()
            }
            Message::FeedScrolled { at_bottom } => {
                self.view_model.messages_mut().set_at_bottom(at_bottom);
                Task::none()
            }
            Message::Shortcut { window, shortcut } => match shortcut {
                Shortcut::Navigate(route) if self.main_window == Some(window) => {
                    self.update(Message::Navigate(route))
                }
                Shortcut::OpenSettings => self.update(Message::OpenSettings),
                Shortcut::GrowMessages if self.main_window == Some(window) => {
                    self.update(Message::ResizeMessages(
                        self.view_model.message_pane_height().saturating_add(16),
                    ))
                }
                Shortcut::ShrinkMessages if self.main_window == Some(window) => {
                    self.update(Message::ResizeMessages(
                        self.view_model.message_pane_height().saturating_sub(16),
                    ))
                }
                Shortcut::Remote(action)
                    if self.main_window == Some(window)
                        && self.view_model.primary_view()
                            == super::view_model::PrimaryView::Remote =>
                {
                    let Some(target) = self.app_state.selected_device() else {
                        return self.publish(
                            super::view_model::MessageSeverity::Warning,
                            super::view_model::MessageSource::MainWindow,
                            "No TV selected. Open Settings to choose a TV.",
                        );
                    };
                    self.update(Message::AttemptRemoteAction(SendRemoteAction::new(
                        target,
                        self.app_state.selection_generation(),
                        action,
                    )))
                }
                _ => Task::none(),
            },
            Message::AttemptRemoteAction(request) => self.handle_remote_action(request),
            Message::TvAddressChanged(value) => {
                self.tv_address = value;
                self.observed = None;
                self.probe_attempt = self.probe_attempt.saturating_add(1);
                self.bump_pair_attempt();
                self.pair_pending = false;
                Task::none()
            }
            Message::DiscoverTv => {
                self.discovery_attempt = self.discovery_attempt.saturating_add(1);
                let attempt = self.discovery_attempt;
                self.candidates.clear();
                self.settings_status = "Searching for Samsung TV advertisements…".to_owned();
                Task::perform(
                    async move { SsdpDiscovery.discover().await },
                    move |result| Message::DiscoveryFinished { attempt, result },
                )
            }
            Message::DiscoveryFinished { attempt, result } => {
                if attempt != self.discovery_attempt {
                    return Task::none();
                }
                let severity = match result {
                    Ok(candidates) => {
                        self.settings_status = if candidates.is_empty() {
                            "No TV advertisements found. Enter the TV address manually.".to_owned()
                        } else {
                            format!(
                                "{} unconfirmed Samsung candidate(s) found. Choose one to probe.",
                                candidates.len()
                            )
                        };
                        self.candidates = candidates;
                        super::view_model::MessageSeverity::Information
                    }
                    Err(DiscoveryError::Permission) => {
                        self.settings_status = "Local-network access was denied. Allow this app to use the local network, then retry discovery or enter the TV address manually.".to_owned();
                        super::view_model::MessageSeverity::Warning
                    }
                    Err(DiscoveryError::Unavailable) => {
                        self.settings_status =
                            "Discovery unavailable. Enter the TV address manually.".to_owned();
                        super::view_model::MessageSeverity::Warning
                    }
                };
                self.publish(
                    severity,
                    super::view_model::MessageSource::SettingsWindow,
                    self.settings_status.clone(),
                )
            }
            Message::UseCandidate(host) => {
                self.tv_address = host.as_str().to_owned();
                self.update(Message::ProbeTv)
            }
            Message::ProbeTv => self.probe_tv(),
            Message::ProbeFinished {
                attempt,
                host,
                result,
            } => {
                if attempt != self.probe_attempt {
                    return Task::none();
                }
                match result {
                    Ok(observation) => {
                        self.observed = Some((host, observation));
                        self.settings_status = "Secure TV endpoint found. Confirm the TV and certificate before Pairing.".to_owned();
                        self.publish(
                            super::view_model::MessageSeverity::Information,
                            super::view_model::MessageSource::SettingsWindow,
                            "Secure TV endpoint found; confirm its certificate in Settings.",
                        )
                    }
                    Err(error) => {
                        self.settings_status = probe_error_message(error).to_owned();
                        self.publish(
                            super::view_model::MessageSeverity::Warning,
                            super::view_model::MessageSource::SettingsWindow,
                            self.settings_status.clone(),
                        )
                    }
                }
            }
            Message::ConfirmAndPair => self.confirm_and_pair(None),
            Message::ConfirmAndRepair => self.confirm_and_pair(self.app_state.selected_device()),
            Message::PairFinished {
                attempt,
                replace,
                result,
            } => self.finish_pairing(attempt, replace, result),
            Message::StaleSetupCleaned(result) => {
                if result.complete() {
                    Task::none()
                } else {
                    self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::SettingsWindow,
                        "Cancelled TV setup left credentials behind. Check Keychain.",
                    )
                }
            }
            Message::ConnectSelected => self.connect_selected(),
            Message::ConnectFinished {
                attempt,
                generation,
                result,
            } => self.finish_connect(attempt, generation, result),
            Message::ForgetSelected => self.forget_selected(),
            Message::ForgetFinished { id, result } => {
                if self.forget_pending != Some(id) {
                    return Task::none();
                }
                self.forget_pending = None;
                if result.record_removed {
                    self.saved_devices.retain(|device| device.id != id);
                }
                self.settings_status = if result.complete() {
                    "TV forgotten and credentials removed.".to_owned()
                } else {
                    "TV removal had partial credential cleanup. Check Keychain for this app's pairing item."
                        .to_owned()
                };
                self.publish(
                    if result.complete() {
                        super::view_model::MessageSeverity::Information
                    } else {
                        super::view_model::MessageSeverity::Warning
                    },
                    super::view_model::MessageSource::SettingsWindow,
                    self.settings_status.clone(),
                )
            }
            Message::SelectSaved(id) => {
                if self.forget_pending.is_some() {
                    return Task::none();
                }
                let Some(service) = self.service.clone() else {
                    return Task::none();
                };
                self.bump_pair_attempt();
                let selected = {
                    let Ok(_guard) = self.setup_guard.lock() else {
                        return Task::none();
                    };
                    service.select_saved(id)
                };
                let Ok(device) = selected else {
                    return self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::SettingsWindow,
                        "Could not select the saved TV.",
                    );
                };
                self.stop_active();
                self.connect_attempt = self.connect_attempt.saturating_add(1);
                if self.app_state.selected_device() == Some(device.id) {
                    self.app_state.restart_selection(device.display());
                } else {
                    self.app_state.select_device(device.display());
                }
                self.app_state.set_verified_actions(
                    self.app_state.selection_generation(),
                    device.verified_actions,
                );
                self.view_model.update_control_state(&self.app_state);
                self.tv_address = device.host.as_str().to_owned();
                Task::done(Message::ConnectSelected)
            }
            Message::SetActionVerified { action, verified } => {
                let (Some(service), Some(id)) =
                    (self.service.as_ref(), self.app_state.selected_device())
                else {
                    return Task::none();
                };
                match service.set_action_verified(id, action, verified) {
                    Ok(actions) => {
                        if let Some(saved) =
                            self.saved_devices.iter_mut().find(|saved| saved.id == id)
                        {
                            saved.verified_actions = actions.clone();
                        }
                        self.app_state
                            .set_verified_actions(self.app_state.selection_generation(), actions);
                        self.view_model.update_control_state(&self.app_state);
                        self.publish(
                            super::view_model::MessageSeverity::Information,
                            super::view_model::MessageSource::SettingsWindow,
                            format!(
                                "{} support updated from your TV observation.",
                                remote_action_label(action)
                            ),
                        )
                    }
                    Err(_) => self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::SettingsWindow,
                        "Could not save verified key support. Retry in Settings.",
                    ),
                }
            }
            Message::SessionEvent {
                generation,
                session_id,
                event,
            } => self.handle_session_event(generation, session_id, event),
        }
    }

    fn probe_tv(&mut self) -> Task<Message> {
        let host = match TvHost::parse(&self.tv_address) {
            Ok(host) => host,
            Err(_) => {
                self.settings_status = "Enter a valid local TV IP address or host name.".to_owned();
                return Task::none();
            }
        };
        self.probe_attempt = self.probe_attempt.saturating_add(1);
        self.bump_pair_attempt();
        self.pair_pending = false;
        let attempt = self.probe_attempt;
        self.observed = None;
        self.settings_status = "Checking the secure TV endpoint on port 8002…".to_owned();
        let probing_host = host.clone();
        Task::perform(
            async move { session::probe_tv(&probing_host).await },
            move |result| Message::ProbeFinished {
                attempt,
                host,
                result,
            },
        )
    }

    fn confirm_and_pair(&mut self, replace: Option<crate::DeviceId>) -> Task<Message> {
        if self.pair_pending || self.forget_pending.is_some() {
            return Task::none();
        }
        let Some((host, observation)) = self.observed.clone() else {
            self.settings_status = "Probe a TV before confirming Pairing.".to_owned();
            return Task::none();
        };
        let Some(service) = self.service.clone() else {
            self.settings_status = "Application storage is unavailable.".to_owned();
            return Task::none();
        };
        self.bump_pair_attempt();
        self.pair_pending = true;
        let attempt = self.pair_attempt;
        let epoch = Arc::clone(&self.pair_epoch);
        let guard = Arc::clone(&self.setup_guard);
        self.settings_status = "Waiting for approval on the physical TV…".to_owned();
        let pin = observation.pin;
        let label = observation.name.unwrap_or_else(|| "Samsung TV".to_owned());
        Task::perform(
            async move {
                let mut live = session::connect(&host, pin, None)
                    .await
                    .map_err(PairFlowError::Session)?;
                let token = live
                    .await_authorized(false)
                    .await
                    .map_err(PairFlowError::Session)?
                    .ok_or(PairFlowError::Session(
                        session::SessionError::PairingTokenMissing,
                    ))?;
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
                            Err(PairFlowError::Setup(
                                crate::application::device_service::SetupError::PartialCleanup,
                            ))
                        };
                    }
                    Ok(device)
                })
                .await
                .map_err(|_| {
                    PairFlowError::Setup(
                        crate::application::device_service::SetupError::PartialCleanup,
                    )
                })??;
                Ok(SessionPackage::new(device, live))
            },
            move |result| Message::PairFinished {
                attempt,
                replace,
                result,
            },
        )
    }

    fn finish_pairing(
        &mut self,
        attempt: u64,
        replace: Option<crate::DeviceId>,
        result: Result<SessionPackage, PairFlowError>,
    ) -> Task<Message> {
        if attempt != self.pair_attempt {
            if replace.is_some() {
                if let Ok(package) = result {
                    if let Some((device, _)) = package.take() {
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
                            self.view_model.update_control_state(&self.app_state);
                        }
                    }
                } else {
                    return Task::none();
                }
                return self.publish(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::SettingsWindow,
                    "Re-pair completed after setup changed. Reconnect the saved TV to use its new credential.",
                );
            }
            if let (Ok(package), Some(service)) = (result, self.service.clone()) {
                if let Some((device, _)) = package.take() {
                    let guard = Arc::clone(&self.setup_guard);
                    return Task::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                let Ok(_guard) = guard.lock() else {
                                    return crate::application::device_service::ForgetResult {
                                        credential_removed: false,
                                        trust_removed: false,
                                        record_removed: false,
                                    };
                                };
                                service.forget(device.id)
                            })
                            .await
                            .unwrap_or(
                                crate::application::device_service::ForgetResult {
                                    credential_removed: false,
                                    trust_removed: false,
                                    record_removed: false,
                                },
                            )
                        },
                        Message::StaleSetupCleaned,
                    );
                }
            }
            return Task::none();
        }
        self.pair_pending = false;
        match result {
            Ok(package) => {
                let Some((device, live)) = package.take() else {
                    return Task::none();
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
                    .set_pairing_state(generation, PairingState::Ready);
                self.app_state
                    .set_connection_state(generation, ConnectionState::Ready);
                self.app_state
                    .set_verified_actions(generation, device.verified_actions.clone());
                self.view_model.update_control_state(&self.app_state);
                self.settings_status =
                    "Paired and connected. Verify supported keys before using Remote View."
                        .to_owned();
                let observe = self.install_session(generation, live);
                Task::batch([
                    observe,
                    self.publish(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::SettingsWindow,
                        "TV paired and connected over secure port 8002.",
                    ),
                ])
            }
            Err(error) => {
                self.settings_status = pair_error_message(error).to_owned();
                self.publish(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::SettingsWindow,
                    self.settings_status.clone(),
                )
            }
        }
    }

    fn connect_selected(&mut self) -> Task<Message> {
        if self.forget_pending.is_some() {
            return Task::none();
        }
        let (Some(service), Some(id)) = (self.service.clone(), self.app_state.selected_device())
        else {
            return Task::none();
        };
        self.stop_active();
        self.bump_pair_attempt();
        self.pair_pending = false;
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        let attempt = self.connect_attempt;
        let pair_epoch = self.pair_attempt;
        let epoch = Arc::clone(&self.pair_epoch);
        let guard = Arc::clone(&self.setup_guard);
        let generation = self.app_state.selection_generation();
        self.app_state
            .set_connection_state(generation, ConnectionState::Connecting);
        self.view_model.update_control_state(&self.app_state);
        self.settings_status = "Reconnecting to the saved TV securely…".to_owned();
        Task::perform(
            async move {
                let material = tokio::task::spawn_blocking({
                    let service = service.clone();
                    move || service.reconnect_material(id)
                })
                .await
                .map_err(|_| {
                    ConnectFlowError::Reconnect(
                        crate::application::device_service::ReconnectError::MissingDevice,
                    )
                })?
                .map_err(ConnectFlowError::Reconnect)?;
                let mut live =
                    session::connect(&material.device.host, material.pin, Some(&material.token))
                        .await
                        .map_err(ConnectFlowError::Session)?;
                if let Some(rotated) = live
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
                Ok(SessionPackage::new(material.device, live))
            },
            move |result| Message::ConnectFinished {
                attempt,
                generation,
                result,
            },
        )
    }

    fn finish_connect(
        &mut self,
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    ) -> Task<Message> {
        if attempt != self.connect_attempt || generation != self.app_state.selection_generation() {
            return Task::none();
        }
        match result {
            Ok(package) => {
                let Some((device, live)) = package.take() else {
                    return Task::none();
                };
                if self.app_state.selected_device() != Some(device.id) {
                    return Task::none();
                }
                self.app_state
                    .set_pairing_state(generation, PairingState::Ready);
                self.app_state
                    .set_connection_state(generation, ConnectionState::Ready);
                let verified_actions = self
                    .saved_devices
                    .iter()
                    .find(|saved| saved.id == device.id)
                    .map(|saved| saved.verified_actions.clone())
                    .unwrap_or(device.verified_actions);
                self.app_state
                    .set_verified_actions(generation, verified_actions);
                self.view_model.update_control_state(&self.app_state);
                self.settings_status = "Connected to the saved TV.".to_owned();
                let observe = self.install_session(generation, live);
                Task::batch([
                    observe,
                    self.publish(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::MainWindow,
                        "Saved TV reconnected securely.",
                    ),
                ])
            }
            Err(error) => {
                self.app_state
                    .set_connection_state(generation, ConnectionState::Failed);
                if matches!(
                    error,
                    ConnectFlowError::Session(session::SessionError::TokenRejected)
                        | ConnectFlowError::Reconnect(
                            crate::application::device_service::ReconnectError::PairingRequired
                        )
                ) {
                    self.app_state
                        .set_pairing_state(generation, PairingState::Failed);
                }
                self.view_model.update_control_state(&self.app_state);
                self.settings_status = connect_error_message(error).to_owned();
                self.publish(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::MainWindow,
                    self.settings_status.clone(),
                )
            }
        }
    }

    fn install_session(&mut self, generation: u64, live: session::Session) -> Task<Message> {
        let active = live.start_actor();
        self.next_session_id = self.next_session_id.saturating_add(1);
        let session_id = self.next_session_id;
        self.active_runtime = Some(ActiveRuntime {
            generation,
            session_id,
            commands: active.commands,
            task: active.task,
        });
        Task::run(
            stream::unfold(active.events, |mut events| async move {
                events.recv().await.map(|event| (event, events))
            }),
            move |event| Message::SessionEvent {
                generation,
                session_id,
                event,
            },
        )
    }

    fn stop_active(&mut self) {
        if let Some(active) = self.active_runtime.take() {
            active.task.abort();
        }
        self.dispatcher.abort_in_flight();
        self.dispatcher.cancel_waiting();
        let _ = self.sync_terminal_results();
    }

    fn forget_selected(&mut self) -> Task<Message> {
        if self.forget_pending.is_some() {
            return Task::none();
        }
        let (Some(service), Some(id)) = (self.service.clone(), self.app_state.selected_device())
        else {
            return Task::none();
        };
        self.stop_active();
        self.bump_pair_attempt();
        self.pair_pending = false;
        self.connect_attempt = self.connect_attempt.saturating_add(1);
        self.app_state.clear_selection();
        self.view_model.update_control_state(&self.app_state);
        self.observed = None;
        self.forget_pending = Some(id);
        self.settings_status = "Removing saved TV and credentials…".to_owned();
        let guard = Arc::clone(&self.setup_guard);
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let Ok(_guard) = guard.lock() else {
                        return crate::application::device_service::ForgetResult {
                            credential_removed: false,
                            trust_removed: false,
                            record_removed: false,
                        };
                    };
                    service.forget(id)
                })
                .await
                .unwrap_or(crate::application::device_service::ForgetResult {
                    credential_removed: false,
                    trust_removed: false,
                    record_removed: false,
                })
            },
            move |result| Message::ForgetFinished { id, result },
        )
    }

    fn bump_pair_attempt(&mut self) {
        self.pair_attempt = self.pair_attempt.saturating_add(1);
        self.pair_epoch.store(self.pair_attempt, Ordering::SeqCst);
    }

    fn handle_remote_action(&mut self, request: SendRemoteAction) -> Task<Message> {
        match self.dispatcher.admit(&self.app_state, request) {
            Admission::Rejected(DispatchRejection::Policy(reason)) => self.publish(
                super::view_model::MessageSeverity::Warning,
                super::view_model::MessageSource::MainWindow,
                format!("Remote action not sent. {}", rejection_message(reason)),
            ),
            Admission::Rejected(DispatchRejection::Busy) => self.publish(
                super::view_model::MessageSeverity::Warning,
                super::view_model::MessageSource::MainWindow,
                "Remote queue is busy. Try again after an outcome appears.",
            ),
            Admission::Queued(id) => {
                let queued = self.publish(
                    super::view_model::MessageSeverity::Information,
                    super::view_model::MessageSource::MainWindow,
                    format!("Request #{} queued.", id.value()),
                );
                Task::batch([queued, self.pump_dispatch()])
            }
        }
    }

    fn pump_dispatch(&mut self) -> Task<Message> {
        let Some(item) = self.dispatcher.start_next(&self.app_state) else {
            return self.sync_terminal_results();
        };
        let stale = self.sync_terminal_results();
        let sent = self
            .active_runtime
            .as_ref()
            .filter(|active| active.generation == self.app_state.selection_generation())
            .is_some_and(|active| {
                active
                    .commands
                    .try_send(SessionCommand::Click {
                        id: item.id,
                        action: item.request.action(),
                    })
                    .is_ok()
            });
        if sent {
            stale
        } else {
            self.dispatcher.finish(
                item.id,
                TerminalOutcome::NotSent(NotSentReason::TransportUnavailable),
            );
            Task::batch([stale, self.sync_terminal_results()])
        }
    }

    fn sync_terminal_results(&mut self) -> Task<Message> {
        let mut follow = false;
        while let Some(result) = self.dispatcher.terminal_results().front().cloned() {
            let outcome = match result.outcome {
                TerminalOutcome::Written => {
                    "written to the TV connection; TV response is unverified."
                }
                TerminalOutcome::Uncertain => {
                    "uncertain after a transport failure; it will not be retried."
                }
                TerminalOutcome::NotSent(NotSentReason::Cancelled) => "cancelled before the write.",
                TerminalOutcome::NotSent(NotSentReason::Policy(_)) => {
                    "not sent because control availability changed."
                }
                TerminalOutcome::NotSent(NotSentReason::TransportUnavailable) => {
                    "not sent; reconnect the TV."
                }
            };
            let text = format!("Request #{} {outcome}", result.id.value());
            self.view_model.add_activity(format!(
                "#{}, {}: {outcome}",
                result.id.value(),
                remote_action_label(result.request.action())
            ));
            follow |= self.view_model.messages_mut().append(
                if matches!(result.outcome, TerminalOutcome::Written) {
                    super::view_model::MessageSeverity::Information
                } else {
                    super::view_model::MessageSeverity::Warning
                },
                super::view_model::MessageSource::MainWindow,
                text,
            );
            self.dispatcher.acknowledge_through(result.id);
        }
        if follow {
            ::iced::widget::operation::snap_to_end(view::MESSAGE_FEED_ID)
        } else {
            Task::none()
        }
    }

    fn handle_session_event(
        &mut self,
        generation: u64,
        session_id: u64,
        event: SessionEvent,
    ) -> Task<Message> {
        if self
            .active_runtime
            .as_ref()
            .is_none_or(|active| active.generation != generation || active.session_id != session_id)
        {
            return Task::none();
        }
        match event {
            SessionEvent::Written(id) | SessionEvent::Uncertain(id) | SessionEvent::NotSent(id) => {
                let outcome = match event {
                    SessionEvent::Written(_) => TerminalOutcome::Written,
                    SessionEvent::Uncertain(_) => TerminalOutcome::Uncertain,
                    _ => TerminalOutcome::NotSent(NotSentReason::TransportUnavailable),
                };
                if !self.dispatcher.finish(id, outcome) {
                    return Task::none();
                }
                Task::batch([self.sync_terminal_results(), self.pump_dispatch()])
            }
            SessionEvent::Disconnected | SessionEvent::TokenRejected => {
                self.active_runtime = None;
                self.dispatcher.abort_in_flight();
                self.dispatcher.cancel_waiting();
                let journal = self.sync_terminal_results();
                self.app_state
                    .set_connection_state(generation, ConnectionState::Failed);
                if matches!(event, SessionEvent::TokenRejected) {
                    self.app_state
                        .set_pairing_state(generation, PairingState::Failed);
                }
                self.view_model.update_control_state(&self.app_state);
                self.settings_status = if matches!(event, SessionEvent::TokenRejected) {
                    "TV rejected the saved token. Re-pair in Settings.".to_owned()
                } else {
                    "TV connection closed. Retry in Settings.".to_owned()
                };
                Task::batch([
                    journal,
                    self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::MainWindow,
                        self.settings_status.clone(),
                    ),
                ])
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.settings_window == Some(window) {
            view::settings_window(view::SettingsView {
                address: &self.tv_address,
                candidates: &self.candidates,
                saved_devices: &self.saved_devices,
                fingerprint: self
                    .observed
                    .as_ref()
                    .map(|(_, observation)| observation.pin.to_hex()),
                observed_name: self
                    .observed
                    .as_ref()
                    .and_then(|(_, observation)| observation.name.clone()),
                observed_model: self
                    .observed
                    .as_ref()
                    .and_then(|(_, observation)| observation.model.clone()),
                status: &self.settings_status,
                selected_label: self
                    .app_state
                    .selected_device_display()
                    .map(|device| device.label()),
                pairing_pending: self.pair_pending,
                forget_pending: self.forget_pending.is_some(),
                verified_actions: crate::RemoteAction::LIVE_ACTIONS
                    .map(|action| (action, self.app_state.is_action_verified(action))),
            })
        } else {
            view::main_window(&self.view_model)
        }
    }

    fn title(&self, window: window::Id) -> String {
        if self.settings_window == Some(window) {
            "Samsung TV Remote Settings".to_owned()
        } else {
            "Samsung TV Remote".to_owned()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_events().map(Message::WindowClosed),
            event::listen_with(shortcut_event),
        ])
    }

    fn publish(
        &mut self,
        severity: super::view_model::MessageSeverity,
        source: super::view_model::MessageSource,
        text: impl Into<String>,
    ) -> Task<Message> {
        if self
            .view_model
            .messages_mut()
            .append(severity, source, text)
        {
            ::iced::widget::operation::snap_to_end(view::MESSAGE_FEED_ID)
        } else {
            Task::none()
        }
    }
}

fn shortcut_event(event: Event, status: event::Status, window: window::Id) -> Option<Message> {
    if status != event::Status::Ignored {
        return None;
    }

    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        modified_key,
        physical_key,
        modifiers,
        text,
        repeat: false,
        ..
    }) = event
    else {
        return None;
    };

    let shifted_plus = modifiers == Modifiers::SHIFT
        && (matches!(modified_key.as_ref(), Key::Character("+"))
            || matches!(
                physical_key,
                keyboard::key::Physical::Code(keyboard::key::Code::Equal)
            )
            || text.as_deref() == Some("+"));
    let shortcut = if shifted_plus {
        Some(Shortcut::Remote(crate::RemoteAction::VolumeUp))
    } else {
        shortcut_for(&key, modifiers)
    };
    shortcut.map(|shortcut| Message::Shortcut { window, shortcut })
}

fn shortcut_for(key: &Key, modifiers: Modifiers) -> Option<Shortcut> {
    if modifiers == Modifiers::NONE {
        return match key.as_ref() {
            Key::Named(key::Named::ArrowUp) => Some(Shortcut::Remote(crate::RemoteAction::Up)),
            Key::Named(key::Named::ArrowDown) => Some(Shortcut::Remote(crate::RemoteAction::Down)),
            Key::Named(key::Named::ArrowLeft) => Some(Shortcut::Remote(crate::RemoteAction::Left)),
            Key::Named(key::Named::ArrowRight) => {
                Some(Shortcut::Remote(crate::RemoteAction::Right))
            }
            Key::Named(key::Named::Enter) => Some(Shortcut::Remote(crate::RemoteAction::Select)),
            Key::Named(key::Named::Escape) => Some(Shortcut::Remote(crate::RemoteAction::Back)),
            Key::Named(key::Named::Home) => Some(Shortcut::Remote(crate::RemoteAction::Home)),
            Key::Character("m") => Some(Shortcut::Remote(crate::RemoteAction::Mute)),
            Key::Character("+") => Some(Shortcut::Remote(crate::RemoteAction::VolumeUp)),
            Key::Character("-") => Some(Shortcut::Remote(crate::RemoteAction::VolumeDown)),
            _ => None,
        };
    }
    if !modifiers.command() || modifiers.alt() {
        return None;
    }

    if modifiers.shift() {
        return match key {
            Key::Named(key::Named::ArrowUp) => Some(Shortcut::GrowMessages),
            Key::Named(key::Named::ArrowDown) => Some(Shortcut::ShrinkMessages),
            _ => None,
        };
    }

    match key.as_ref() {
        Key::Character("1") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Remote)),
        Key::Character("2") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Sources)),
        Key::Character("3") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Apps)),
        Key::Character("4") => Some(Shortcut::Navigate(
            super::view_model::PrimaryView::TextInput,
        )),
        Key::Character(",") => Some(Shortcut::OpenSettings),
        _ => None,
    }
}

fn probe_error_message(error: session::SessionError) -> &'static str {
    match error {
        session::SessionError::Target(_) => {
            "This TV address does not resolve to a local network target."
        }
        session::SessionError::Timeout => {
            "TV probe timed out. Check local-network permission and try again."
        }
        session::SessionError::Offline => {
            "TV was not reachable on secure port 8002. Check its address and network."
        }
        session::SessionError::Tls => "Secure connection failed. Check the TV and retry.",
        _ => "TV probe failed. Retry from Settings.",
    }
}

fn pair_error_message(error: PairFlowError) -> &'static str {
    match error {
        PairFlowError::Cancelled => "TV setup was cancelled before it could be saved.",
        PairFlowError::Session(session::SessionError::PairingDenied) => "TV pairing was denied. Approve this Mac on the physical TV and retry.",
        PairFlowError::Session(session::SessionError::CertificateChanged) => "TV certificate changed. Probe and confirm the TV again.",
        PairFlowError::Session(session::SessionError::Timeout) => "TV pairing timed out. Check the TV prompt and retry.",
        PairFlowError::Session(session::SessionError::PairingTokenMissing) => "TV accepted the connection without a pairing token. Remove this client on the TV, then retry Pairing.",
        PairFlowError::Session(session::SessionError::WebSocket) => "TV rejected the secure remote channel. Confirm its address and retry.",
        PairFlowError::Session(session::SessionError::Protocol) => "TV sent an invalid pairing response. Retry Pairing.",
        PairFlowError::Session(session::SessionError::Tls) => "Secure TV handshake failed. Probe the TV again.",
        PairFlowError::Session(session::SessionError::Offline) => "TV connection closed during Pairing. Check the network and retry.",
        PairFlowError::Setup(crate::application::device_service::SetupError::PartialCleanup) => "Pairing could not be saved and credential cleanup was incomplete. Check Keychain.",
        PairFlowError::Setup(_) => "Pairing succeeded but could not be saved securely. Retry after checking Keychain access.",
        PairFlowError::Session(_) => "TV pairing failed. Check the TV and retry.",
    }
}

fn connect_error_message(error: ConnectFlowError) -> &'static str {
    match error {
        ConnectFlowError::Session(session::SessionError::CertificateChanged) => {
            "TV certificate changed. Reconfirm its identity and re-pair."
        }
        ConnectFlowError::Session(session::SessionError::TokenRejected) => {
            "TV rejected the saved token. Re-pair in Settings."
        }
        ConnectFlowError::Session(session::SessionError::Offline) => {
            "Saved TV is offline or unreachable. Check the network and retry."
        }
        ConnectFlowError::Session(session::SessionError::Timeout) => {
            "TV connection timed out. Check local-network permission and retry."
        }
        ConnectFlowError::Reconnect(
            crate::application::device_service::ReconnectError::HostChanged,
        ) => "Saved TV address changed. Confirm the TV and re-pair.",
        ConnectFlowError::Reconnect(
            crate::application::device_service::ReconnectError::PairingRequired,
        ) => "Saved TV token is missing. Re-pair in Settings.",
        ConnectFlowError::CredentialSave => {
            "TV returned a new token, but Keychain could not save it. Re-pair in Settings."
        }
        ConnectFlowError::Session(session::SessionError::WebSocket) => {
            "TV rejected the saved remote channel. Re-pair in Settings."
        }
        ConnectFlowError::Session(session::SessionError::Protocol) => {
            "TV sent an invalid reconnect response. Re-pair in Settings."
        }
        ConnectFlowError::Session(session::SessionError::Tls) => {
            "Secure TV handshake failed. Probe the TV and retry."
        }
        _ => "Saved TV connection failed. Retry or re-pair in Settings.",
    }
}

pub fn run() -> ::iced::Result {
    ::iced::daemon(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .run()
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(980.0, 760.0),
        min_size: Some(Size::new(700.0, 560.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

fn settings_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(680.0, 480.0),
        min_size: Some(Size::new(520.0, 360.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::device::DeviceRepository;

    fn saved_device(id: crate::DeviceId) -> SavedDevice {
        SavedDevice {
            id,
            label: "TV".to_owned(),
            host: TvHost::parse("tv.local").unwrap(),
            verified_actions: vec![crate::RemoteAction::Up],
        }
    }

    #[test]
    fn reselecting_the_same_tv_starts_a_new_disconnected_generation() {
        let directory = tempfile::tempdir().unwrap();
        let device = saved_device(crate::DeviceId::new(7));
        let repository = LocalDeviceRepository::new(directory.path());
        repository.save(&device).unwrap();
        let (mut app, _) = App::new();
        app.service = Some(Arc::new(DeviceService::new(
            repository,
            KeychainSecretStore,
            LocalTrustStore::new(directory.path()),
        )));
        app.saved_devices.push(device.clone());
        app.app_state.select_device(device.display());
        let previous = app.app_state.selection_generation();
        app.app_state
            .set_pairing_state(previous, PairingState::Ready);
        app.app_state
            .set_connection_state(previous, ConnectionState::Ready);
        app.app_state
            .set_verified_actions(previous, [crate::RemoteAction::Up]);

        let _ = app.update(Message::SelectSaved(device.id));

        assert_eq!(app.app_state.selection_generation(), previous + 1);
        assert_eq!(
            app.app_state.connection_state(),
            ConnectionState::NotConnected
        );
        assert_eq!(
            app.app_state.control_status(crate::RemoteAction::Up),
            crate::application::ControlStatus::Unavailable(
                crate::application::RemoteActionRejection::PairingRequired
            )
        );
    }

    #[test]
    fn verified_key_changes_refresh_the_saved_device_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let device = saved_device(crate::DeviceId::new(8));
        let repository = LocalDeviceRepository::new(directory.path());
        repository.save(&device).unwrap();
        let (mut app, _) = App::new();
        app.service = Some(Arc::new(DeviceService::new(
            repository,
            KeychainSecretStore,
            LocalTrustStore::new(directory.path()),
        )));
        app.saved_devices.push(device.clone());
        app.app_state.select_device(device.display());

        let _ = app.update(Message::SetActionVerified {
            action: crate::RemoteAction::Down,
            verified: true,
        });

        assert!(app.app_state.is_action_verified(crate::RemoteAction::Down));
        assert!(app.saved_devices[0]
            .verified_actions
            .contains(&crate::RemoteAction::Down));
    }

    #[test]
    fn selecting_a_saved_tv_waits_for_pending_forget_to_finish() {
        let directory = tempfile::tempdir().unwrap();
        let device = saved_device(crate::DeviceId::new(8));
        let repository = LocalDeviceRepository::new(directory.path());
        repository.save(&device).unwrap();
        let (mut app, _) = App::new();
        app.service = Some(Arc::new(DeviceService::new(
            repository,
            KeychainSecretStore,
            LocalTrustStore::new(directory.path()),
        )));
        app.forget_pending = Some(crate::DeviceId::new(7));

        let _ = app.update(Message::SelectSaved(device.id));

        assert_eq!(app.app_state.selected_device(), None);
        assert_eq!(app.forget_pending, Some(crate::DeviceId::new(7)));
    }

    #[test]
    fn discovery_permission_error_offers_retry_and_manual_entry() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::DiscoverTv);
        let _ = app.update(Message::DiscoveryFinished {
            attempt: app.discovery_attempt,
            result: Err(DiscoveryError::Permission),
        });

        assert!(app.settings_status.contains("retry discovery"));
        assert!(app.settings_status.contains("manually"));
        assert!(app.candidates.is_empty());
        assert_eq!(
            app.view_model.messages().entries().back().unwrap().severity,
            super::super::view_model::MessageSeverity::Warning
        );
    }

    #[tokio::test]
    async fn delayed_disconnect_from_old_session_cannot_fail_new_session() {
        let (mut app, _) = App::new();
        let device = saved_device(crate::DeviceId::new(7));
        app.app_state.select_device(device.display());
        let generation = app.app_state.selection_generation();
        app.app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.app_state
            .set_connection_state(generation, ConnectionState::Ready);
        let (commands, _) = mpsc::channel(1);
        app.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 2,
            commands,
            task: tokio::spawn(async {}),
        });

        let _ = app.update(Message::SessionEvent {
            generation,
            session_id: 1,
            event: SessionEvent::Disconnected,
        });
        assert_eq!(app.app_state.connection_state(), ConnectionState::Ready);
        assert_eq!(
            app.active_runtime.as_ref().map(|active| active.session_id),
            Some(2)
        );

        let _ = app.update(Message::SessionEvent {
            generation,
            session_id: 2,
            event: SessionEvent::Disconnected,
        });
        assert_eq!(app.app_state.connection_state(), ConnectionState::Failed);
    }

    #[test]
    fn remote_shortcut_and_button_intent_share_dispatch_outcomes() {
        let (mut app, _) = App::new();
        let window = window::Id::unique();
        app.main_window = Some(window);
        let device = saved_device(crate::DeviceId::new(7));
        app.app_state.select_device(device.display());
        let generation = app.app_state.selection_generation();
        app.app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.app_state
            .set_connection_state(generation, ConnectionState::Ready);
        app.app_state
            .set_verified_actions(generation, [crate::RemoteAction::Up]);

        let _ = app.update(Message::Shortcut {
            window,
            shortcut: Shortcut::Remote(crate::RemoteAction::Up),
        });
        let _ = app.update(Message::AttemptRemoteAction(SendRemoteAction::new(
            device.id,
            generation,
            crate::RemoteAction::Up,
        )));

        let activity = app.view_model.activity();
        assert_eq!(activity.len(), 2);
        assert!(activity
            .iter()
            .all(|entry| { entry.contains("Up: not sent; reconnect the TV") }));
    }

    #[tokio::test]
    async fn written_session_result_is_reported_without_claiming_tv_response() {
        let (mut app, _) = App::new();
        let device = saved_device(crate::DeviceId::new(7));
        app.app_state.select_device(device.display());
        let generation = app.app_state.selection_generation();
        app.app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.app_state
            .set_connection_state(generation, ConnectionState::Ready);
        app.app_state
            .set_verified_actions(generation, [crate::RemoteAction::Up]);
        let request = SendRemoteAction::new(device.id, generation, crate::RemoteAction::Up);
        let Admission::Queued(id) = app.dispatcher.admit(&app.app_state, request) else {
            panic!("not queued")
        };
        assert_eq!(app.dispatcher.start_next(&app.app_state).unwrap().id, id);
        let (commands, _) = mpsc::channel(1);
        app.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 1,
            commands,
            task: tokio::spawn(async {}),
        });

        let _ = app.update(Message::SessionEvent {
            generation,
            session_id: 1,
            event: SessionEvent::Written(id),
        });

        let activity = app.view_model.activity().back().unwrap();
        assert!(activity.contains("written to the TV connection"));
        assert!(activity.contains("TV response is unverified"));
        assert_eq!(app.dispatcher.pending_ids().count(), 0);
    }

    #[test]
    fn opening_settings_is_idempotent_until_the_window_closes() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::OpenSettings);
        let settings_id = app.settings_window;

        let _ = app.update(Message::OpenSettings);
        assert_eq!(app.settings_window, settings_id);

        let _ = app.update(Message::WindowClosed(settings_id.expect("opened")));
        assert!(app.settings_window.is_none());
    }

    #[test]
    fn closing_settings_preserves_main_navigation_and_message_state() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::Navigate(
            super::super::view_model::PrimaryView::Apps,
        ));
        let _ = app.publish(
            super::super::view_model::MessageSeverity::Information,
            super::super::view_model::MessageSource::MainWindow,
            "A safe presentation message",
        );
        let _ = app.update(Message::OpenSettings);
        let settings_id = app.settings_window.expect("settings window");

        let _ = app.update(Message::WindowClosed(settings_id));

        assert_eq!(
            app.view_model.primary_view(),
            super::super::view_model::PrimaryView::Apps
        );
        assert_eq!(app.view_model.messages().entries().len(), 3);
    }

    #[test]
    fn delayed_window_open_event_does_not_restore_closed_settings() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::OpenSettings);
        let settings_id = app.settings_window.expect("settings window");

        let _ = app.update(Message::WindowClosed(settings_id));
        let _ = app.update(Message::SettingsWindowOpened(settings_id));

        assert!(app.settings_window.is_none());
    }

    #[test]
    fn keyboard_shortcuts_require_command_and_match_routes_and_resize() {
        let command = Modifiers::COMMAND;
        assert_eq!(
            shortcut_for(&Key::Character("4".into()), command),
            Some(Shortcut::Navigate(
                super::super::view_model::PrimaryView::TextInput
            ))
        );
        assert_eq!(
            shortcut_for(&Key::Character(",".into()), command),
            Some(Shortcut::OpenSettings)
        );
        assert_eq!(
            shortcut_for(&Key::Named(key::Named::ArrowUp), command | Modifiers::SHIFT),
            Some(Shortcut::GrowMessages)
        );
        assert_eq!(
            shortcut_for(&Key::Character("1".into()), Modifiers::NONE),
            None
        );
    }

    #[test]
    fn remote_shortcuts_map_to_semantic_actions_only_for_ignored_nonrepeat_keys() {
        assert_eq!(
            shortcut_for(&Key::Named(key::Named::ArrowUp), Modifiers::NONE),
            Some(Shortcut::Remote(crate::RemoteAction::Up))
        );
        assert_eq!(
            shortcut_for(&Key::Named(key::Named::Enter), Modifiers::NONE),
            Some(Shortcut::Remote(crate::RemoteAction::Select))
        );
        assert_eq!(
            shortcut_for(&Key::Character("m".into()), Modifiers::NONE),
            Some(Shortcut::Remote(crate::RemoteAction::Mute))
        );
        let event = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Named(key::Named::ArrowUp),
            modified_key: Key::Named(key::Named::ArrowUp),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::ArrowUp),
            location: keyboard::Location::Standard,
            modifiers: Modifiers::NONE,
            text: None,
            repeat: true,
        });
        let window = window::Id::unique();
        assert!(shortcut_event(event.clone(), event::Status::Ignored, window).is_none());
        assert!(shortcut_event(event, event::Status::Captured, window).is_none());

        let shifted_plus = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character("=".into()),
            modified_key: Key::Character("+".into()),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Equal),
            location: keyboard::Location::Standard,
            modifiers: Modifiers::SHIFT,
            text: Some("+".into()),
            repeat: false,
        });
        assert!(matches!(
            shortcut_event(shifted_plus, event::Status::Ignored, window),
            Some(Message::Shortcut {
                shortcut: Shortcut::Remote(crate::RemoteAction::VolumeUp),
                ..
            })
        ));

        let shifted_equal_without_modified_plus = Event::Keyboard(keyboard::Event::KeyPressed {
            key: Key::Character("=".into()),
            modified_key: Key::Character("=".into()),
            physical_key: keyboard::key::Physical::Code(keyboard::key::Code::Equal),
            location: keyboard::Location::Standard,
            modifiers: Modifiers::SHIFT,
            text: None,
            repeat: false,
        });
        assert!(matches!(
            shortcut_event(
                shifted_equal_without_modified_plus,
                event::Status::Ignored,
                window
            ),
            Some(Message::Shortcut {
                shortcut: Shortcut::Remote(crate::RemoteAction::VolumeUp),
                ..
            })
        ));
    }

    #[test]
    fn navigation_shortcut_from_settings_does_not_change_main_route() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::OpenMainWindow);
        let _ = app.update(Message::OpenSettings);
        let settings_id = app.settings_window.expect("settings window");

        let _ = app.update(Message::Shortcut {
            window: settings_id,
            shortcut: Shortcut::Navigate(super::super::view_model::PrimaryView::Sources),
        });

        assert_eq!(
            app.view_model.primary_view(),
            super::super::view_model::PrimaryView::Remote
        );
    }

    #[test]
    fn remote_shortcut_from_settings_does_not_emit_a_tv_request() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::OpenSettings);
        let settings_id = app.settings_window.expect("settings window");
        let before = app.view_model.messages().entries().len();
        let _ = app.update(Message::Shortcut {
            window: settings_id,
            shortcut: Shortcut::Remote(crate::RemoteAction::Up),
        });
        assert_eq!(app.view_model.messages().entries().len(), before);
        assert_eq!(app.dispatcher.pending_ids().count(), 0);
    }

    #[test]
    fn remote_intent_rejection_is_safe_and_does_not_change_control_state() {
        let (mut app, _) = App::new();
        let request = crate::SendRemoteAction::new(
            crate::DeviceId::new(42),
            0,
            crate::RemoteAction::PowerToggle,
        );

        let _ = app.update(Message::AttemptRemoteAction(request));

        let entry = app
            .view_model
            .messages()
            .entries()
            .back()
            .expect("rejection message");
        assert_eq!(
            entry.severity,
            super::super::view_model::MessageSeverity::Warning
        );
        assert_eq!(
            entry.text,
            "Remote action not sent. No TV selected. Open Settings to choose a TV."
        );
        assert!(!entry.text.contains("dev_"));
        assert_eq!(app.app_state.selected_device(), None);
    }

    #[test]
    fn eligible_remote_intent_does_not_claim_it_was_sent() {
        let (mut app, _) = App::new();
        let _ = app.app_state.select_device(crate::DeviceDisplay::new(
            crate::DeviceId::new(42),
            "Studio TV",
        ));
        let generation = app.app_state.selection_generation();
        let _ = app
            .app_state
            .set_pairing_state(generation, crate::application::PairingState::Ready);
        let _ = app
            .app_state
            .set_connection_state(generation, crate::application::ConnectionState::Ready);
        let _ = app
            .app_state
            .set_verified_actions(generation, [crate::RemoteAction::Up]);
        app.view_model.update_control_state(&app.app_state);

        let request = crate::SendRemoteAction::new(
            crate::DeviceId::new(42),
            generation,
            crate::RemoteAction::Up,
        );
        let _ = app.update(Message::AttemptRemoteAction(request));

        let messages = app.view_model.messages().entries();
        assert_eq!(messages.len(), 2);
        assert!(messages.back().unwrap().text.contains("not sent"));
        assert!(!messages.iter().any(|entry| entry.text.contains("written")));
        assert!(app.view_model.control_state().remote_actions_enabled());
    }
}
