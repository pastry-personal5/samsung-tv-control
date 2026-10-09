use super::view_model::PrimaryView;
use crate::application::tv_address::TvHost;
use crate::application::tv_control_coordinator::{
    ConnectFlowError, PairFlowError, SessionPackage, WakeFailure,
};
use crate::application::tv_discovery::DiscoveryError;
use crate::application::tv_session::{ProbeObservation, TvSessionError, TvSessionEvent};
use crate::application::tv_setup_service::ForgetResult;
use crate::application::wake::WakeInterface;
use crate::application::wake_transport::WakeSendError;
use crate::application::SendRemoteAction;
use crate::domain::DeviceId;
use crate::domain::RemoteAction;
use ::iced::window;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsPage {
    #[default]
    Discovery,
    WakeOnLan,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenMainWindow,
    Navigate(PrimaryView),
    OpenSettings,
    SelectSettingsPage(SettingsPage),
    MainWindowOpened(window::Id),
    SettingsWindowOpened(window::Id),
    WindowClosed(window::Id),
    ResizeMessages(u16),
    FeedScrolled {
        at_bottom: bool,
    },
    Shortcut {
        window: window::Id,
        shortcut: Shortcut,
    },
    /// Typed remote action intent; disabled controls never emit it.
    AttemptRemoteAction(SendRemoteAction),
    PowerToggle(SendRemoteAction),
    Wake,
    CancelWake,
    WakeTick,
    PowerProbeFinished {
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    },
    WakeSent {
        attempt: u64,
        generation: u64,
        device: DeviceId,
        result: Result<(), WakeSendError>,
    },
    WakeReconnected {
        attempt: u64,
        generation: u64,
        device: DeviceId,
        result: Result<SessionPackage, WakeFailure>,
    },
    WakeWiredChanged(String),
    WakeWifiChanged(String),
    WakeInterfaceSelected(WakeInterface),
    WakeInterfaceCleared,
    TvAddressChanged(String),
    DiscoverTv,
    DiscoveryFinished {
        attempt: u64,
        result: Result<Vec<TvHost>, DiscoveryError>,
    },
    UseCandidate(TvHost),
    ProbeTv,
    ProbeFinished {
        attempt: u64,
        host: TvHost,
        result: Result<ProbeObservation, TvSessionError>,
    },
    ConfirmAndPair,
    ConfirmAndRepair,
    PairFinished {
        attempt: u64,
        replace: Option<DeviceId>,
        result: Result<SessionPackage, PairFlowError>,
    },
    StaleSetupCleaned(ForgetResult),
    ConnectSelected,
    ConnectFinished {
        attempt: u64,
        generation: u64,
        result: Result<SessionPackage, ConnectFlowError>,
    },
    ForgetSelected,
    ForgetFinished {
        id: DeviceId,
        result: ForgetResult,
    },
    SelectSaved(DeviceId),
    SessionEvent {
        generation: u64,
        session_id: u64,
        event: TvSessionEvent,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    Navigate(PrimaryView),
    OpenSettings,
    Remote(RemoteAction),
}
