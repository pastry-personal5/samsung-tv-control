use super::view_model::PrimaryView;
use crate::application::tv_address::TvHost;
use crate::application::tv_control_coordinator::{ConnectFlowError, PairFlowError, SessionPackage};
use crate::application::tv_discovery::DiscoveryError;
use crate::application::tv_session::{ProbeObservation, TvSessionError, TvSessionEvent};
use crate::application::tv_setup_service::ForgetResult;
use crate::application::SendRemoteAction;
use crate::domain::DeviceId;
use crate::domain::RemoteAction;
use ::iced::window;

#[derive(Debug, Clone)]
pub enum Message {
    OpenMainWindow,
    Navigate(PrimaryView),
    OpenSettings,
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
    SetActionVerified {
        action: RemoteAction,
        verified: bool,
    },
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
    GrowMessages,
    ShrinkMessages,
    Remote(RemoteAction),
}
