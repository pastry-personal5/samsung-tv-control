use super::ui_message::{Message, Shortcut};
use super::view;
use super::view_model::{rejection_message, ViewModel};
use crate::application::device_repository::RepositoryError;
use crate::application::remote_dispatcher::{
    Admission, DispatchRejection, NotSentReason, TerminalOutcome,
};
use crate::application::tv_control_coordinator::{
    AppServices, ConnectCompletion, ConnectFlowError, PairCompletion, PairFlowError,
    PairStartError, PowerProbeCompletion, PowerStart, SessionImpact, SessionPackage,
    TvControlCoordinator, WakeFailure, WakeStage,
};
use crate::application::tv_discovery::DiscoveryError;
use crate::application::tv_session::{TvSessionError, TvSessionEvent};
use crate::application::wake::{WakeConfiguration, WakeInterface};
use crate::application::SendRemoteAction;
use ::iced::keyboard::{self, key, Key, Modifiers};
use ::iced::{event, window, Element, Event, Size, Subscription, Task, Theme};

pub struct App {
    main_window: Option<window::Id>,
    settings_window: Option<window::Id>,
    view_model: ViewModel,
    coordinator: TvControlCoordinator,
    tv_address: String,
    settings_status: String,
    wake_wired: String,
    wake_wifi: String,
    wake_active: Option<WakeInterface>,
    wake_tick: bool,
}

impl App {
    #[cfg(test)]
    fn new() -> (Self, Task<Message>) {
        Self::new_with_services(AppServices::without_adapters())
    }

    fn new_with_services(services: AppServices) -> (Self, Task<Message>) {
        let coordinator = TvControlCoordinator::new(services);
        let view_model = ViewModel::new(&coordinator.app_state);
        (
            Self {
                main_window: None,
                settings_window: None,
                view_model,
                coordinator,
                tv_address: String::new(),
                settings_status: "Enter a local TV address to begin secure setup.".to_owned(),
                wake_wired: String::new(),
                wake_wifi: String::new(),
                wake_active: None,
                wake_tick: false,
            },
            Task::done(Message::OpenMainWindow),
        )
    }

    fn boot(services: AppServices) -> (Self, Task<Message>) {
        let (mut app, open) = Self::new_with_services(services);
        let Some(restore) = app.coordinator.restore() else {
            app.settings_status = "Application Support is unavailable on this Mac.".to_owned();
            return (app, open);
        };
        let mut tasks = vec![open];
        if restore.list_error {
            app.settings_status = "Saved TV list needs attention.".to_owned();
        }
        match restore.selected {
            Ok(Some(device)) => {
                app.tv_address = device.host.as_str().to_owned();
                app.load_wake_draft(&device);
                app.view_model
                    .update_control_state(&app.coordinator.app_state);
                app.settings_status = "Restoring the selected TV securely.".to_owned();
                tasks.push(Task::done(Message::ConnectSelected));
            }
            Ok(None) => {}
            Err(RepositoryError::Corrupt) => app.settings_status =
                "Saved TV settings cannot be read. Back up the old settings file and pair again."
                    .to_owned(),
            Err(RepositoryError::Unavailable) => {
                app.settings_status = "Saved TV settings need attention.".to_owned()
            }
        }
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
                Shortcut::Remote(action)
                    if self.main_window == Some(window)
                        && self.view_model.primary_view()
                            == super::view_model::PrimaryView::Remote =>
                {
                    let Some(target) = self.coordinator.app_state.selected_device() else {
                        return self.publish(
                            super::view_model::MessageSeverity::Warning,
                            super::view_model::MessageSource::MainWindow,
                            "No TV selected. Open Settings to choose a TV.",
                        );
                    };
                    let request = SendRemoteAction::new(
                        target,
                        self.coordinator.app_state.selection_generation(),
                        action,
                    );
                    if action == crate::RemoteAction::PowerToggle {
                        self.update(Message::PowerToggle(request))
                    } else {
                        self.update(Message::AttemptRemoteAction(request))
                    }
                }
                _ => Task::none(),
            },
            Message::AttemptRemoteAction(request) => self.handle_remote_action(request),
            Message::PowerToggle(request) => self.power_toggle(request),
            Message::Wake => self.begin_wake(),
            Message::CancelWake => {
                self.coordinator.cancel_wake();
                self.publish(
                    super::view_model::MessageSeverity::Information,
                    super::view_model::MessageSource::MainWindow,
                    "Wake cancelled. A packet already sent cannot be recalled.",
                )
            }
            Message::WakeTick => {
                self.wake_tick = !self.wake_tick;
                Task::none()
            }
            Message::PowerProbeFinished {
                attempt,
                generation,
                result,
            } => self.finish_power_probe(attempt, generation, result),
            Message::WakeSent {
                attempt,
                generation,
                device,
                result,
            } => self.finish_wake_send(attempt, generation, device, result),
            Message::WakeReconnected {
                attempt,
                generation,
                device,
                result,
            } => self.finish_wake_reconnect(attempt, generation, device, result),
            Message::WakeWiredChanged(value) => {
                self.wake_wired = value;
                self.auto_save_wake_configuration()
            }
            Message::WakeWifiChanged(value) => {
                self.wake_wifi = value;
                self.auto_save_wake_configuration()
            }
            Message::WakeInterfaceSelected(value) => {
                self.wake_active = Some(value);
                self.auto_save_wake_configuration()
            }
            Message::WakeInterfaceCleared => {
                self.wake_active = None;
                self.auto_save_wake_configuration()
            }
            Message::SaveWakeConfiguration => self.save_wake_configuration(true),
            Message::TvAddressChanged(value) => {
                self.tv_address = value;
                self.coordinator.address_changed();
                Task::none()
            }
            Message::DiscoverTv => {
                let Some(plan) = self.coordinator.begin_discovery() else {
                    return Task::none();
                };
                let attempt = plan.attempt;
                self.settings_status = "Searching for Samsung TV advertisements…".to_owned();
                Task::perform(plan.run(), move |result| Message::DiscoveryFinished {
                    attempt,
                    result,
                })
            }
            Message::DiscoveryFinished { attempt, result } => {
                if !self.coordinator.finish_discovery(attempt, &result) {
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
                if !self.coordinator.finish_probe(attempt, host, &result) {
                    return Task::none();
                }
                match result {
                    Ok(_) => {
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
            Message::ConfirmAndRepair => {
                self.confirm_and_pair(self.coordinator.app_state.selected_device())
            }
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
                if !self.coordinator.finish_forget(id, result) {
                    return Task::none();
                }
                self.settings_status = if result.complete() {
                    "TV forgotten and credentials removed.".to_owned()
                } else {
                    "TV removal had partial credential cleanup. Check Keychain for this app's pairing item."
                        .to_owned()
                };
                if result.record_removed {
                    self.wake_wired.clear();
                    self.wake_wifi.clear();
                    self.wake_active = None;
                }
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
                if self.coordinator.forget_pending.is_some() {
                    return Task::none();
                }
                let Ok(device) = self.coordinator.select_saved(id) else {
                    return self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::SettingsWindow,
                        "Could not select the saved TV.",
                    );
                };
                let _ = self.sync_terminal_results();
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.tv_address = device.host.as_str().to_owned();
                self.load_wake_draft(&device);
                Task::done(Message::ConnectSelected)
            }
            Message::SessionEvent {
                generation,
                session_id,
                event,
            } => self.handle_session_event(generation, session_id, event),
        }
    }

    fn probe_tv(&mut self) -> Task<Message> {
        let plan = match self.coordinator.begin_probe(&self.tv_address) {
            Ok(plan) => plan,
            Err(_) => {
                self.settings_status = "Enter a valid local TV IP address or host name.".to_owned();
                return Task::none();
            }
        };
        let attempt = plan.attempt;
        let host = plan.host.clone();
        self.settings_status = "Checking the secure TV endpoint on port 8002…".to_owned();
        Task::perform(plan.run(), move |result| Message::ProbeFinished {
            attempt,
            host,
            result,
        })
    }

    fn confirm_and_pair(&mut self, replace: Option<crate::DeviceId>) -> Task<Message> {
        let plan = match self.coordinator.begin_pair(replace) {
            Ok(plan) => plan,
            Err(PairStartError::ProbeRequired) => {
                self.settings_status = "Probe a TV before confirming Pairing.".to_owned();
                return Task::none();
            }
            Err(PairStartError::StorageUnavailable) => {
                self.settings_status = "Application storage is unavailable.".to_owned();
                return Task::none();
            }
            Err(PairStartError::Busy | PairStartError::TransportUnavailable) => {
                return Task::none();
            }
        };
        let attempt = plan.attempt;
        let replace = plan.replace;
        self.settings_status = "Waiting for approval on the physical TV…".to_owned();
        Task::perform(plan.run(), move |result| Message::PairFinished {
            attempt,
            replace,
            result,
        })
    }

    fn finish_pairing(
        &mut self,
        attempt: u64,
        replace: Option<crate::DeviceId>,
        result: Result<SessionPackage, PairFlowError>,
    ) -> Task<Message> {
        match self.coordinator.finish_pair(attempt, replace, result) {
            PairCompletion::Ignored => Task::none(),
            PairCompletion::StaleRepaired => {
                let _ = self.sync_terminal_results();
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.publish(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::SettingsWindow,
                    "Re-pair completed after setup changed. Reconnect the saved TV to use its new credential.",
                )
            }
            PairCompletion::StaleNew(plan) => Task::perform(plan.run(), Message::StaleSetupCleaned),
            PairCompletion::Connected {
                generation,
                connection,
            } => {
                if let Some(device) = self
                    .coordinator
                    .app_state
                    .selected_device()
                    .and_then(|id| {
                        self.coordinator
                            .saved_devices
                            .iter()
                            .find(|device| device.id == id)
                    })
                    .cloned()
                {
                    self.load_wake_draft(&device);
                }
                let _ = self.sync_terminal_results();
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.settings_status =
                    "Paired and connected. Verify supported keys before using Remote View."
                        .to_owned();
                let observe = self.install_session(generation, connection);
                Task::batch([
                    observe,
                    self.publish(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::SettingsWindow,
                        "TV paired and connected over secure port 8002.",
                    ),
                ])
            }
            PairCompletion::Failed(error) => {
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
        let Some(plan) = self.coordinator.begin_connect() else {
            return Task::none();
        };
        let attempt = plan.attempt;
        let generation = plan.generation;
        let _ = self.sync_terminal_results();
        self.view_model
            .update_control_state(&self.coordinator.app_state);
        self.settings_status = "Reconnecting to the saved TV securely…".to_owned();
        Task::perform(plan.run(), move |result| Message::ConnectFinished {
            attempt,
            generation,
            result,
        })
    }

    fn finish_connect(
        &mut self,
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    ) -> Task<Message> {
        match self.coordinator.finish_connect(attempt, generation, result) {
            ConnectCompletion::Ignored => Task::none(),
            ConnectCompletion::Connected { connection } => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.settings_status = "Connected to the saved TV.".to_owned();
                let observe = self.install_session(generation, connection);
                Task::batch([
                    observe,
                    self.publish(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::MainWindow,
                        "Saved TV reconnected securely.",
                    ),
                ])
            }
            ConnectCompletion::Failed(error) => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.settings_status = connect_error_message(error).to_owned();
                self.publish(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::MainWindow,
                    self.settings_status.clone(),
                )
            }
        }
    }

    fn install_session(
        &mut self,
        generation: u64,
        live: Box<dyn crate::application::tv_session::TvConnection>,
    ) -> Task<Message> {
        let (session_id, events) = self.coordinator.install_session(generation, live);
        Task::run(events, move |event| Message::SessionEvent {
            generation,
            session_id,
            event,
        })
    }

    fn forget_selected(&mut self) -> Task<Message> {
        let Some(plan) = self.coordinator.begin_forget() else {
            return Task::none();
        };
        let id = plan.id;
        let _ = self.sync_terminal_results();
        self.view_model
            .update_control_state(&self.coordinator.app_state);
        self.settings_status = "Removing saved TV and credentials…".to_owned();
        Task::perform(plan.run(), move |result| Message::ForgetFinished {
            id,
            result,
        })
    }

    fn handle_remote_action(&mut self, request: SendRemoteAction) -> Task<Message> {
        match self.coordinator.admit_remote(request) {
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
            Admission::Queued(_) => {
                let queued = self.publish(
                    super::view_model::MessageSeverity::Information,
                    super::view_model::MessageSource::MainWindow,
                    "Remote request queued.",
                );
                Task::batch([queued, self.pump_dispatch()])
            }
        }
    }

    fn power_toggle(&mut self, request: SendRemoteAction) -> Task<Message> {
        let plan =
            match self.coordinator.begin_power_toggle(&request) {
                PowerStart::Toggle => return self.handle_remote_action(request),
                PowerStart::Probe(plan) => plan,
                PowerStart::MissingConfiguration => return self.publish(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::MainWindow,
                    "Set an active Wake MAC in TV Settings before using Power while disconnected.",
                ),
                PowerStart::Busy | PowerStart::Stale => return Task::none(),
                PowerStart::Unavailable => {
                    return self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::MainWindow,
                        "Power service is unavailable. Check the selected TV in Settings.",
                    )
                }
            };
        let attempt = plan.attempt;
        let generation = plan.generation;
        self.view_model
            .select_view(super::view_model::PrimaryView::Power);
        self.view_model
            .update_control_state(&self.coordinator.app_state);
        Task::perform(plan.run(), move |result| Message::PowerProbeFinished {
            attempt,
            generation,
            result,
        })
    }

    fn finish_power_probe(
        &mut self,
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    ) -> Task<Message> {
        match self
            .coordinator
            .finish_power_probe(attempt, generation, result)
        {
            PowerProbeCompletion::Ignored => Task::none(),
            PowerProbeCompletion::Connected { connection } => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                Task::batch([self.install_session(generation, connection),
                    self.publish(super::view_model::MessageSeverity::Information, super::view_model::MessageSource::MainWindow,
                        "TV was already reachable. Paired remote channel is ready; no wake packet was sent.")])
            }
            PowerProbeCompletion::Wake(plan) => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.run_wake_plan(plan)
            }
            PowerProbeCompletion::Failed(_) => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.publish(super::view_model::MessageSeverity::Warning, super::view_model::MessageSource::MainWindow,
                    "Power connection check needs attention. Review pairing and connection in TV Settings; no wake packet was sent.")
            }
        }
    }

    fn begin_wake(&mut self) -> Task<Message> {
        let Some(plan) = self.coordinator.begin_wake() else {
            return self.publish(
                super::view_model::MessageSeverity::Warning,
                super::view_model::MessageSource::MainWindow,
                "Wake cannot start. Check the selected TV and Wake configuration in Settings.",
            );
        };
        self.run_wake_plan(plan)
    }

    fn run_wake_plan(
        &mut self,
        plan: crate::application::tv_control_coordinator::WakePlan,
    ) -> Task<Message> {
        let attempt = plan.attempt;
        let generation = plan.generation;
        let device = plan.device;
        self.view_model
            .select_view(super::view_model::PrimaryView::Power);
        Task::perform(plan.run(), move |result| Message::WakeSent {
            attempt,
            generation,
            device,
            result,
        })
    }

    fn finish_wake_send(
        &mut self,
        attempt: u64,
        generation: u64,
        device: crate::DeviceId,
        result: Result<(), crate::application::wake_transport::WakeSendError>,
    ) -> Task<Message> {
        let current = self.coordinator.wake_attempt == attempt
            && self.coordinator.app_state.selection_generation() == generation
            && self.coordinator.app_state.selected_device() == Some(device);
        let Some(plan) = self
            .coordinator
            .finish_wake_send(attempt, generation, device, result)
        else {
            if current
                && matches!(
                    self.coordinator.wake_stage,
                    WakeStage::Failed(WakeFailure::Local(_))
                )
            {
                return self.publish(super::view_model::MessageSeverity::Warning, super::view_model::MessageSource::MainWindow,
                    "Magic packet could not be sent on this local network. Check permission and route in TV Settings.");
            }
            return Task::none();
        };
        let message = self.publish(
            super::view_model::MessageSeverity::Information,
            super::view_model::MessageSource::MainWindow,
            "Magic packet sent once. Delivery and physical TV state are unconfirmed.",
        );
        Task::batch([
            message,
            Task::perform(plan.run(), move |result| Message::WakeReconnected {
                attempt,
                generation,
                device,
                result,
            }),
        ])
    }

    fn finish_wake_reconnect(
        &mut self,
        attempt: u64,
        generation: u64,
        device: crate::DeviceId,
        result: Result<SessionPackage, WakeFailure>,
    ) -> Task<Message> {
        match self
            .coordinator
            .finish_wake_reconnect(attempt, generation, device, result)
        {
            ConnectCompletion::Ignored => Task::none(),
            ConnectCompletion::Connected { connection } => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                Task::batch([
                    self.install_session(generation, connection),
                    self.publish(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::MainWindow,
                        "Paired remote channel is ready. Physical panel state is not measured.",
                    ),
                ])
            }
            ConnectCompletion::Failed(_) => {
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.publish(super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::MainWindow,
                    "Wake did not establish the paired remote channel. Check Power View for the result.")
            }
        }
    }

    fn load_wake_draft(&mut self, device: &crate::application::device_repository::SavedDevice) {
        self.wake_wired = device
            .wake
            .wired
            .map(|value| value.to_string())
            .unwrap_or_default();
        self.wake_wifi = device
            .wake
            .wifi
            .map(|value| value.to_string())
            .unwrap_or_default();
        self.wake_active = device.wake.active;
    }

    fn auto_save_wake_configuration(&mut self) -> Task<Message> {
        if self.coordinator.app_state.selected_device().is_none() {
            return Task::none();
        }
        let parse = |value: &str| {
            if value.is_empty() {
                Ok(None)
            } else {
                value.parse().map(Some)
            }
        };
        let (Ok(wired), Ok(wifi)) = (parse(&self.wake_wired), parse(&self.wake_wifi)) else {
            return Task::none();
        };
        let wake = WakeConfiguration {
            wired,
            wifi,
            active: self.wake_active,
        };
        if wake.active.is_some() && wake.active_mac().is_none() {
            return Task::none();
        }
        self.persist_wake_configuration(wake, false)
    }

    fn save_wake_configuration(&mut self, announce: bool) -> Task<Message> {
        let parse = |value: &str| {
            if value.is_empty() {
                Ok(None)
            } else {
                value.parse().map(Some)
            }
        };
        let (Ok(wired), Ok(wifi)) = (parse(&self.wake_wired), parse(&self.wake_wifi)) else {
            self.settings_status =
                "Enter each MAC as six hexadecimal octets, for example 02:11:22:33:44:55."
                    .to_owned();
            return Task::none();
        };
        let wake = WakeConfiguration {
            wired,
            wifi,
            active: self.wake_active,
        };
        if wake.active.is_some() && wake.active_mac().is_none() {
            self.settings_status = "Enter a MAC for the selected Wake interface.".to_owned();
            return Task::none();
        }
        self.persist_wake_configuration(wake, announce)
    }

    fn persist_wake_configuration(
        &mut self,
        wake: WakeConfiguration,
        announce: bool,
    ) -> Task<Message> {
        let Ok(device) = self.coordinator.save_wake_configuration(wake) else {
            self.settings_status = "Could not save Wake configuration.".to_owned();
            return Task::none();
        };
        self.load_wake_draft(&device);
        self.settings_status = "Wake configuration saved.".to_owned();
        if announce {
            self.publish(
                super::view_model::MessageSeverity::Information,
                super::view_model::MessageSource::SettingsWindow,
                "Wake configuration saved for the selected TV.",
            )
        } else {
            Task::none()
        }
    }

    fn pump_dispatch(&mut self) -> Task<Message> {
        self.coordinator.pump_dispatch();
        self.sync_terminal_results()
    }

    fn sync_terminal_results(&mut self) -> Task<Message> {
        let mut follow = false;
        for result in self.coordinator.terminal_results_snapshot() {
            let outcome = match result.outcome {
                TerminalOutcome::Written => None,
                TerminalOutcome::Uncertain => {
                    Some("uncertain after a transport failure; it will not be retried.")
                }
                TerminalOutcome::NotSent(NotSentReason::Cancelled) => {
                    Some("cancelled before the write.")
                }
                TerminalOutcome::NotSent(NotSentReason::Policy(_)) => {
                    Some("not sent because control availability changed.")
                }
                TerminalOutcome::NotSent(NotSentReason::TransportUnavailable) => {
                    Some("not sent; reconnect the TV.")
                }
            };
            if let Some(outcome) = outcome {
                let text = format!("Remote request {outcome}");
                follow |= self.view_model.messages_mut().append(
                    super::view_model::MessageSeverity::Warning,
                    super::view_model::MessageSource::MainWindow,
                    text,
                );
            }
            self.coordinator.acknowledge_terminal_through(result.id);
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
        event: TvSessionEvent,
    ) -> Task<Message> {
        match self
            .coordinator
            .process_session_event(generation, session_id, event)
        {
            SessionImpact::Ignored => Task::none(),
            SessionImpact::RemoteResult => {
                Task::batch([self.sync_terminal_results(), self.pump_dispatch()])
            }
            SessionImpact::Disconnected | SessionImpact::TokenRejected => {
                let journal = self.sync_terminal_results();
                self.view_model
                    .update_control_state(&self.coordinator.app_state);
                self.settings_status = if matches!(event, TvSessionEvent::TokenRejected) {
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
                candidates: &self.coordinator.candidates,
                saved_devices: &self.coordinator.saved_devices,
                fingerprint: self
                    .coordinator
                    .observed
                    .as_ref()
                    .map(|(_, observation)| observation.pin.to_hex()),
                observed_name: self
                    .coordinator
                    .observed
                    .as_ref()
                    .and_then(|(_, observation)| observation.name.clone()),
                observed_model: self
                    .coordinator
                    .observed
                    .as_ref()
                    .and_then(|(_, observation)| observation.model.clone()),
                status: &self.settings_status,
                selected_label: self
                    .coordinator
                    .app_state
                    .selected_device_display()
                    .map(|device| device.label()),
                pairing_pending: self.coordinator.pair_pending,
                forget_pending: self.coordinator.forget_pending.is_some(),
                wake_wired: &self.wake_wired,
                wake_wifi: &self.wake_wifi,
                wake_active: self.wake_active,
            })
        } else {
            view::main_window(
                &self.view_model,
                self.coordinator.wake_stage,
                self.coordinator.selected_wake_configured(),
                self.wake_tick,
            )
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
        let mut subscriptions = vec![
            window::close_events().map(Message::WindowClosed),
            event::listen_with(shortcut_event),
        ];
        if matches!(
            self.coordinator.wake_stage,
            WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
        ) {
            subscriptions.push(
                ::iced::time::every(std::time::Duration::from_millis(500))
                    .map(|_| Message::WakeTick),
            );
        }
        Subscription::batch(subscriptions)
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
            Key::Named(key::Named::Enter) => Some(Shortcut::Remote(crate::RemoteAction::Enter)),
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
        return None;
    }

    match key.as_ref() {
        Key::Character("1") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Power)),
        Key::Character("2") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Remote)),
        Key::Character("3") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Sources)),
        Key::Character("4") => Some(Shortcut::Navigate(super::view_model::PrimaryView::Apps)),
        Key::Character("5") => Some(Shortcut::Navigate(
            super::view_model::PrimaryView::TextInput,
        )),
        Key::Character(",") => Some(Shortcut::OpenSettings),
        _ => None,
    }
}

fn probe_error_message(error: TvSessionError) -> &'static str {
    match error {
        TvSessionError::Target(_) => "This TV address does not resolve to a local network target.",
        TvSessionError::Timeout => {
            "TV probe timed out. Check local-network permission and try again."
        }
        TvSessionError::Offline => {
            "TV was not reachable on secure port 8002. Check its address and network."
        }
        TvSessionError::Tls => "Secure connection failed. Check the TV and retry.",
        _ => "TV probe failed. Retry from Settings.",
    }
}

fn pair_error_message(error: PairFlowError) -> &'static str {
    match error {
        PairFlowError::Cancelled => "TV setup was cancelled before it could be saved.",
        PairFlowError::Session(TvSessionError::PairingDenied) => "TV pairing was denied. Approve this Mac on the physical TV and retry.",
        PairFlowError::Session(TvSessionError::CertificateChanged) => "TV certificate changed. Probe and confirm the TV again.",
        PairFlowError::Session(TvSessionError::Timeout) => "TV pairing timed out. Check the TV prompt and retry.",
        PairFlowError::Session(TvSessionError::PairingTokenMissing) => "TV accepted the connection without a pairing token. Remove this client on the TV, then retry Pairing.",
        PairFlowError::Session(TvSessionError::RemoteChannel) => "TV rejected the secure remote channel. Confirm its address and retry.",
        PairFlowError::Session(TvSessionError::Protocol) => "TV sent an invalid pairing response. Retry Pairing.",
        PairFlowError::Session(TvSessionError::Tls) => "Secure TV handshake failed. Probe the TV again.",
        PairFlowError::Session(TvSessionError::Offline) => "TV connection closed during Pairing. Check the network and retry.",
        PairFlowError::Setup(crate::application::tv_setup_service::SetupError::PartialCleanup) => "Pairing could not be saved and credential cleanup was incomplete. Check Keychain.",
        PairFlowError::Setup(_) => "Pairing succeeded but could not be saved securely. Retry after checking Keychain access.",
        PairFlowError::Session(_) => "TV pairing failed. Check the TV and retry.",
    }
}

fn connect_error_message(error: ConnectFlowError) -> &'static str {
    match error {
        ConnectFlowError::Session(TvSessionError::CertificateChanged) => {
            "TV certificate changed. Reconfirm its identity and re-pair."
        }
        ConnectFlowError::Session(TvSessionError::TokenRejected) => {
            "TV rejected the saved token. Re-pair in Settings."
        }
        ConnectFlowError::Session(TvSessionError::Offline) => {
            "Saved TV is offline or unreachable. Check the network and retry."
        }
        ConnectFlowError::Session(TvSessionError::Timeout) => {
            "TV connection timed out. Check local-network permission and retry."
        }
        ConnectFlowError::Reconnect(
            crate::application::tv_setup_service::ReconnectError::HostChanged,
        ) => "Saved TV address changed. Confirm the TV and re-pair.",
        ConnectFlowError::Reconnect(
            crate::application::tv_setup_service::ReconnectError::PairingRequired,
        ) => "Saved TV token is missing. Re-pair in Settings.",
        ConnectFlowError::CredentialSave => {
            "TV returned a new token, but Keychain could not save it. Re-pair in Settings."
        }
        ConnectFlowError::Session(TvSessionError::RemoteChannel) => {
            "TV rejected the saved remote channel. Re-pair in Settings."
        }
        ConnectFlowError::Session(TvSessionError::Protocol) => {
            "TV sent an invalid reconnect response. Re-pair in Settings."
        }
        ConnectFlowError::Session(TvSessionError::Tls) => {
            "Secure TV handshake failed. Probe the TV and retry."
        }
        _ => "Saved TV connection failed. Retry or re-pair in Settings.",
    }
}

pub fn run(services: AppServices) -> ::iced::Result {
    ::iced::daemon(move || App::boot(services.clone()), App::update, App::view)
        .title(App::title)
        .theme(dark_theme)
        .subscription(App::subscription)
        .run()
}

fn dark_theme(_: &App, _: window::Id) -> Theme {
    Theme::Dark
}

fn main_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1100.0, 760.0),
        min_size: Some(Size::new(1100.0, 760.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

fn settings_window_settings() -> window::Settings {
    window::Settings {
        size: Size::new(1000.0, 660.0),
        min_size: Some(Size::new(1000.0, 660.0)),
        position: window::Position::Centered,
        ..window::Settings::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::certificate_trust::CertificatePin;
    use crate::application::credential_store::{PairingToken, SecretError};
    use crate::application::device_repository::{RepositoryError, SavedDevice};
    use crate::application::tv_address::TvHost;
    use crate::application::tv_control_coordinator::ActiveRuntime;
    use crate::application::tv_session::TvSessionControl;
    use crate::application::tv_setup_service::{
        ForgetResult, ReconnectError, ReconnectMaterial, SetupError, TvSetupPort,
    };
    use crate::application::wake_transport::{WakeFuture, WakePermit, WakeTransport};
    use crate::application::{ConnectionState, PairingState};
    use std::sync::{Arc, Mutex};

    struct FakeSetup {
        device: Mutex<SavedDevice>,
        list_failure: bool,
    }

    impl FakeSetup {
        fn new(device: SavedDevice) -> Self {
            Self {
                device: Mutex::new(device),
                list_failure: false,
            }
        }

        fn with_list_failure(device: SavedDevice) -> Self {
            Self {
                device: Mutex::new(device),
                list_failure: true,
            }
        }
    }

    impl TvSetupPort for FakeSetup {
        fn saved_devices(&self) -> Result<Vec<SavedDevice>, RepositoryError> {
            if self.list_failure {
                Err(RepositoryError::Unavailable)
            } else {
                Ok(vec![self.device.lock().unwrap().clone()])
            }
        }

        fn selected_device(&self) -> Result<Option<SavedDevice>, RepositoryError> {
            Ok(Some(self.device.lock().unwrap().clone()))
        }

        fn select_saved(&self, id: crate::DeviceId) -> Result<SavedDevice, RepositoryError> {
            let device = self.device.lock().unwrap().clone();
            (device.id == id)
                .then_some(device)
                .ok_or(RepositoryError::Corrupt)
        }

        fn commit_pairing(
            &self,
            _: &str,
            _: TvHost,
            _: CertificatePin,
            _: PairingToken,
        ) -> Result<SavedDevice, SetupError> {
            Err(SetupError::Preferences(RepositoryError::Unavailable))
        }

        fn repair(
            &self,
            _: crate::DeviceId,
            _: &str,
            _: TvHost,
            _: CertificatePin,
            _: PairingToken,
        ) -> Result<SavedDevice, SetupError> {
            Err(SetupError::Preferences(RepositoryError::Unavailable))
        }

        fn reconnect_material(
            &self,
            _: crate::DeviceId,
        ) -> Result<ReconnectMaterial, ReconnectError> {
            Err(ReconnectError::MissingDevice)
        }

        fn save_rotated_token(
            &self,
            _: crate::DeviceId,
            _: &PairingToken,
        ) -> Result<(), SecretError> {
            Err(SecretError::Unavailable)
        }

        fn save_wake_configuration(
            &self,
            id: crate::DeviceId,
            wake: crate::application::wake::WakeConfiguration,
        ) -> Result<SavedDevice, RepositoryError> {
            let mut device = self.device.lock().unwrap();
            if device.id != id {
                return Err(RepositoryError::Corrupt);
            }
            device.wake = wake;
            Ok(device.clone())
        }

        fn forget(&self, _: crate::DeviceId) -> ForgetResult {
            ForgetResult {
                credential_removed: true,
                trust_removed: true,
                record_removed: true,
            }
        }
    }

    struct TestControl;

    struct FakeWake;

    impl WakeTransport for FakeWake {
        fn send_once(&self, _: TvHost, _: crate::domain::MacAddress, _: WakePermit) -> WakeFuture {
            Box::pin(async { Ok(()) })
        }
    }

    struct RecordingWake {
        sends: Arc<Mutex<Vec<(String, crate::domain::MacAddress)>>>,
    }

    impl WakeTransport for RecordingWake {
        fn send_once(
            &self,
            host: TvHost,
            mac: crate::domain::MacAddress,
            permit: WakePermit,
        ) -> WakeFuture {
            let sends = self.sends.clone();
            Box::pin(async move {
                permit.send_if_current(|| {
                    sends.lock().unwrap().push((host.as_str().to_owned(), mac));
                    Ok(())
                })
            })
        }
    }

    impl TvSessionControl for TestControl {
        fn try_click(
            &self,
            _: crate::application::remote_dispatcher::RequestId,
            _: crate::RemoteAction,
        ) -> bool {
            true
        }

        fn abort(&mut self) {}
    }

    fn saved_device(id: crate::DeviceId) -> SavedDevice {
        SavedDevice {
            id,
            label: "TV".to_owned(),
            host: TvHost::parse("tv.local").unwrap(),
            wake: crate::application::wake::WakeConfiguration::default(),
        }
    }

    #[test]
    fn selected_tv_remains_wake_configured_when_list_load_fails() {
        let mut device = saved_device(crate::DeviceId::new(16));
        device.wake = WakeConfiguration {
            wired: Some("02:11:22:33:44:55".parse().unwrap()),
            wifi: None,
            active: Some(WakeInterface::Wired),
        };
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::with_list_failure(device.clone())));

        let summary = app.coordinator.restore().unwrap();

        assert!(summary.list_error);
        assert_eq!(app.coordinator.saved_devices, vec![device]);
        assert!(app.coordinator.selected_wake_configured());
    }

    #[test]
    fn saving_wake_configuration_adds_missing_cached_selected_tv() {
        let device = saved_device(crate::DeviceId::new(17));
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::new(device.clone())));
        app.coordinator.app_state.select_device(device.display());
        let wake = WakeConfiguration {
            wired: Some("02:11:22:33:44:55".parse().unwrap()),
            wifi: None,
            active: Some(WakeInterface::Wired),
        };

        app.coordinator.save_wake_configuration(wake).unwrap();

        assert_eq!(app.coordinator.saved_devices.len(), 1);
        assert!(app.coordinator.selected_wake_configured());
    }

    #[test]
    fn valid_wake_mac_input_is_persisted_without_a_message_entry() {
        let device = saved_device(crate::DeviceId::new(18));
        let setup = Arc::new(FakeSetup::new(device.clone()));
        let (mut app, _) = App::new();
        app.coordinator.service = Some(setup.clone());
        app.coordinator.app_state.select_device(device.display());
        let messages_before = app.view_model.messages().entries().len();

        let _ = app.update(Message::WakeWiredChanged("02:11:22:33:44:55".to_owned()));

        let saved = setup.device.lock().unwrap();
        assert_eq!(
            saved.wake.wired.map(|mac| mac.to_string()).as_deref(),
            Some("02:11:22:33:44:55")
        );
        assert_eq!(app.view_model.messages().entries().len(), messages_before);
    }

    #[tokio::test]
    async fn manual_wake_uses_only_the_active_mac_once() {
        let mut device = saved_device(crate::DeviceId::new(19));
        device.wake = WakeConfiguration {
            wired: Some("02:11:22:33:44:55".parse().unwrap()),
            wifi: Some("04:11:22:33:44:55".parse().unwrap()),
            active: Some(WakeInterface::WiFi),
        };
        let sends = Arc::new(Mutex::new(Vec::new()));
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::new(device.clone())));
        app.coordinator.gateway = Some(Arc::new(
            crate::infrastructure::samsung::session::SamsungGateway,
        ));
        app.coordinator.wake_transport = Some(Arc::new(RecordingWake {
            sends: sends.clone(),
        }));
        app.coordinator.saved_devices.push(device.clone());
        app.coordinator.app_state.select_device(device.display());

        let plan = app.coordinator.begin_wake().unwrap();
        assert!(plan.run().await.is_ok());

        assert_eq!(
            sends.lock().unwrap().as_slice(),
            &[("tv.local".to_owned(), "04:11:22:33:44:55".parse().unwrap())]
        );
    }

    #[test]
    fn disconnected_power_opens_power_and_starts_one_short_connection_check() {
        let mut device = saved_device(crate::DeviceId::new(12));
        device.wake = WakeConfiguration {
            wired: Some("02:11:22:33:44:55".parse().unwrap()),
            wifi: None,
            active: Some(WakeInterface::Wired),
        };
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::new(device.clone())));
        app.coordinator.gateway = Some(Arc::new(
            crate::infrastructure::samsung::session::SamsungGateway,
        ));
        app.coordinator.saved_devices.push(device.clone());
        app.coordinator.app_state.select_device(device.display());
        let generation = app.coordinator.app_state.selection_generation();
        let request =
            SendRemoteAction::new(device.id, generation, crate::RemoteAction::PowerToggle);
        let _ = app.update(Message::PowerToggle(request.clone()));
        assert_eq!(
            app.view_model.primary_view(),
            super::super::view_model::PrimaryView::Power
        );
        assert_eq!(app.coordinator.wake_stage, WakeStage::CheckingConnection);
        let first_attempt = app.coordinator.connect_attempt;
        let _ = app.update(Message::PowerToggle(request));
        assert_eq!(app.coordinator.connect_attempt, first_attempt);
    }

    #[test]
    fn connected_power_keeps_remote_view_and_queues_key_power() {
        let device = saved_device(crate::DeviceId::new(13));
        let (mut app, _) = App::new();
        app.coordinator.app_state.select_device(device.display());
        let generation = app.coordinator.app_state.selection_generation();
        app.coordinator
            .app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.coordinator
            .app_state
            .set_connection_state(generation, ConnectionState::Ready);
        app.coordinator.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 1,
            control: Box::new(TestControl),
        });
        let _ = app.update(Message::PowerToggle(SendRemoteAction::new(
            device.id,
            generation,
            crate::RemoteAction::PowerToggle,
        )));
        assert_eq!(
            app.view_model.primary_view(),
            super::super::view_model::PrimaryView::Remote
        );
        assert_eq!(app.coordinator.dispatcher.pending_ids().count(), 1);
    }

    #[test]
    fn power_probe_wakes_only_after_an_unreachable_result() {
        let mut device = saved_device(crate::DeviceId::new(14));
        device.wake = WakeConfiguration {
            wired: Some("02:11:22:33:44:55".parse().unwrap()),
            wifi: None,
            active: Some(WakeInterface::Wired),
        };
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::new(device.clone())));
        app.coordinator.gateway = Some(Arc::new(
            crate::infrastructure::samsung::session::SamsungGateway,
        ));
        app.coordinator.wake_transport = Some(Arc::new(FakeWake));
        app.coordinator.saved_devices.push(device.clone());
        app.coordinator.app_state.select_device(device.display());
        let generation = app.coordinator.app_state.selection_generation();
        let request =
            SendRemoteAction::new(device.id, generation, crate::RemoteAction::PowerToggle);
        let _ = app.update(Message::PowerToggle(request));
        let attempt = app.coordinator.connect_attempt;
        let _ = app.update(Message::PowerProbeFinished {
            attempt,
            generation,
            result: Err(ConnectFlowError::Session(TvSessionError::Offline)),
        });
        assert_eq!(app.coordinator.wake_stage, WakeStage::PacketSending);
        assert_eq!(
            app.view_model.control_state().connection.label,
            "Connection needs attention"
        );
        assert_eq!(
            app.view_model.primary_view(),
            super::super::view_model::PrimaryView::Power
        );

        let wake_attempt = app.coordinator.wake_attempt;
        let _ = app.update(Message::WakeSent {
            attempt: wake_attempt,
            generation,
            device: device.id,
            result: Ok(()),
        });
        let _ = app.update(Message::WakeReconnected {
            attempt: wake_attempt,
            generation,
            device: device.id,
            result: Err(WakeFailure::Timeout),
        });
        assert_eq!(
            app.view_model.control_state().connection.label,
            "Connection needs attention"
        );
    }

    #[test]
    fn reselecting_the_same_tv_starts_a_new_disconnected_generation() {
        let device = saved_device(crate::DeviceId::new(7));
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::new(device.clone())));
        app.coordinator.saved_devices.push(device.clone());
        app.coordinator.app_state.select_device(device.display());
        let previous = app.coordinator.app_state.selection_generation();
        app.coordinator
            .app_state
            .set_pairing_state(previous, PairingState::Ready);
        app.coordinator
            .app_state
            .set_connection_state(previous, ConnectionState::Ready);
        let _ = app.update(Message::SelectSaved(device.id));

        assert_eq!(
            app.coordinator.app_state.selection_generation(),
            previous + 1
        );
        assert_eq!(
            app.coordinator.app_state.connection_state(),
            ConnectionState::NotConnected
        );
        assert_eq!(
            app.coordinator
                .app_state
                .control_status(crate::RemoteAction::Up),
            crate::application::ControlStatus::Unavailable(
                crate::application::RemoteActionRejection::PairingRequired
            )
        );
    }

    #[test]
    fn selecting_a_saved_tv_waits_for_pending_forget_to_finish() {
        let device = saved_device(crate::DeviceId::new(8));
        let (mut app, _) = App::new();
        app.coordinator.service = Some(Arc::new(FakeSetup::new(device.clone())));
        app.coordinator.forget_pending = Some(crate::DeviceId::new(7));

        let _ = app.update(Message::SelectSaved(device.id));

        assert_eq!(app.coordinator.app_state.selected_device(), None);
        assert_eq!(
            app.coordinator.forget_pending,
            Some(crate::DeviceId::new(7))
        );
    }

    #[test]
    fn discovery_permission_error_offers_retry_and_manual_entry() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::DiscoverTv);
        let _ = app.update(Message::DiscoveryFinished {
            attempt: app.coordinator.discovery_attempt,
            result: Err(DiscoveryError::Permission),
        });

        assert!(app.settings_status.contains("retry discovery"));
        assert!(app.settings_status.contains("manually"));
        assert!(app.coordinator.candidates.is_empty());
        assert_eq!(
            app.view_model.messages().entries().back().unwrap().severity,
            super::super::view_model::MessageSeverity::Warning
        );
    }

    #[test]
    fn discovered_candidates_survive_closing_and_reopening_settings() {
        let (mut app, _) = App::new();
        let _ = app.update(Message::DiscoverTv);
        let candidate = TvHost::parse("192.168.1.2").unwrap();
        let _ = app.update(Message::DiscoveryFinished {
            attempt: app.coordinator.discovery_attempt,
            result: Ok(vec![candidate.clone()]),
        });
        let _ = app.update(Message::OpenSettings);
        let settings_id = app.settings_window.expect("settings window");
        let _ = app.update(Message::WindowClosed(settings_id));
        let _ = app.update(Message::OpenSettings);

        assert_eq!(app.coordinator.candidates, vec![candidate]);
        assert!(app.settings_window.is_some());
    }

    #[tokio::test]
    async fn delayed_disconnect_from_old_session_cannot_fail_new_session() {
        let (mut app, _) = App::new();
        let device = saved_device(crate::DeviceId::new(7));
        app.coordinator.app_state.select_device(device.display());
        let generation = app.coordinator.app_state.selection_generation();
        app.coordinator
            .app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.coordinator
            .app_state
            .set_connection_state(generation, ConnectionState::Ready);
        app.coordinator.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 2,
            control: Box::new(TestControl),
        });

        let _ = app.update(Message::SessionEvent {
            generation,
            session_id: 1,
            event: TvSessionEvent::Disconnected,
        });
        assert_eq!(
            app.coordinator.app_state.connection_state(),
            ConnectionState::Ready
        );
        assert_eq!(
            app.coordinator
                .active_runtime
                .as_ref()
                .map(|active| active.session_id),
            Some(2)
        );

        let _ = app.update(Message::SessionEvent {
            generation,
            session_id: 2,
            event: TvSessionEvent::Disconnected,
        });
        assert_eq!(
            app.coordinator.app_state.connection_state(),
            ConnectionState::Failed
        );
    }

    #[test]
    fn remote_shortcut_and_button_intent_share_dispatch_outcomes() {
        let (mut app, _) = App::new();
        let window = window::Id::unique();
        app.main_window = Some(window);
        let device = saved_device(crate::DeviceId::new(7));
        app.coordinator.app_state.select_device(device.display());
        let generation = app.coordinator.app_state.selection_generation();
        app.coordinator
            .app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.coordinator
            .app_state
            .set_connection_state(generation, ConnectionState::Ready);
        let _ = app.update(Message::Shortcut {
            window,
            shortcut: Shortcut::Remote(crate::RemoteAction::Up),
        });
        let _ = app.update(Message::AttemptRemoteAction(SendRemoteAction::new(
            device.id,
            generation,
            crate::RemoteAction::Up,
        )));

        assert!(app
            .view_model
            .messages()
            .entries()
            .iter()
            .any(|entry| entry.text.contains("not sent; reconnect the TV")));
    }

    #[tokio::test]
    async fn written_session_result_is_acknowledged_without_global_message() {
        let (mut app, _) = App::new();
        let device = saved_device(crate::DeviceId::new(7));
        app.coordinator.app_state.select_device(device.display());
        let generation = app.coordinator.app_state.selection_generation();
        app.coordinator
            .app_state
            .set_pairing_state(generation, PairingState::Ready);
        app.coordinator
            .app_state
            .set_connection_state(generation, ConnectionState::Ready);
        let request = SendRemoteAction::new(device.id, generation, crate::RemoteAction::Up);
        let Admission::Queued(id) = app
            .coordinator
            .dispatcher
            .admit(&app.coordinator.app_state, request)
        else {
            panic!("not queued")
        };
        assert_eq!(
            app.coordinator
                .dispatcher
                .start_next(&app.coordinator.app_state)
                .unwrap()
                .id,
            id
        );
        app.coordinator.active_runtime = Some(ActiveRuntime {
            generation,
            session_id: 1,
            control: Box::new(TestControl),
        });

        let messages_before = app.view_model.messages().entries().len();
        let _ = app.update(Message::SessionEvent {
            generation,
            session_id: 1,
            event: TvSessionEvent::Written(id),
        });

        assert_eq!(app.view_model.messages().entries().len(), messages_before);
        assert_eq!(app.coordinator.dispatcher.pending_ids().count(), 0);
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
    fn keyboard_shortcuts_require_command_and_match_routes() {
        let command = Modifiers::COMMAND;
        let routes = [
            ("1", super::super::view_model::PrimaryView::Power),
            ("2", super::super::view_model::PrimaryView::Remote),
            ("3", super::super::view_model::PrimaryView::Sources),
            ("4", super::super::view_model::PrimaryView::Apps),
            ("5", super::super::view_model::PrimaryView::TextInput),
        ];
        for (key, route) in routes {
            assert_eq!(
                shortcut_for(&Key::Character(key.into()), command),
                Some(Shortcut::Navigate(route))
            );
        }
        assert_eq!(
            shortcut_for(&Key::Character(",".into()), command),
            Some(Shortcut::OpenSettings)
        );
        assert_eq!(
            shortcut_for(&Key::Named(key::Named::ArrowUp), command | Modifiers::SHIFT),
            None
        );
        assert_eq!(
            shortcut_for(&Key::Character("1".into()), Modifiers::NONE),
            None
        );
    }

    #[test]
    fn windows_use_the_approved_dark_layout_dimensions() {
        assert_eq!(
            main_window_settings().min_size,
            Some(Size::new(1100.0, 760.0))
        );
        assert_eq!(
            settings_window_settings().min_size,
            Some(Size::new(1000.0, 660.0))
        );
        let (app, _) = App::new();
        assert_eq!(dark_theme(&app, window::Id::unique()), Theme::Dark);
    }

    #[test]
    fn remote_shortcuts_map_to_semantic_actions_only_for_ignored_nonrepeat_keys() {
        assert_eq!(
            shortcut_for(&Key::Named(key::Named::ArrowUp), Modifiers::NONE),
            Some(Shortcut::Remote(crate::RemoteAction::Up))
        );
        assert_eq!(
            shortcut_for(&Key::Named(key::Named::Enter), Modifiers::NONE),
            Some(Shortcut::Remote(crate::RemoteAction::Enter))
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
        assert_eq!(app.coordinator.dispatcher.pending_ids().count(), 0);
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
        assert_eq!(app.coordinator.app_state.selected_device(), None);
    }

    #[test]
    fn eligible_remote_intent_does_not_claim_it_was_sent() {
        let (mut app, _) = App::new();
        let _ = app
            .coordinator
            .app_state
            .select_device(crate::DeviceDisplay::new(
                crate::DeviceId::new(42),
                "Studio TV",
            ));
        let generation = app.coordinator.app_state.selection_generation();
        let _ = app
            .coordinator
            .app_state
            .set_pairing_state(generation, crate::application::PairingState::Ready);
        let _ = app
            .coordinator
            .app_state
            .set_connection_state(generation, crate::application::ConnectionState::Ready);
        app.view_model
            .update_control_state(&app.coordinator.app_state);

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
