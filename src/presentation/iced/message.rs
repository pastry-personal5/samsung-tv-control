use super::view_model::PrimaryView;
use crate::application::device::SavedDevice;
use crate::application::device_service::{ForgetResult, ReconnectError, SetupError};
use crate::application::discovery::DiscoveryError;
use crate::application::target::TvHost;
use crate::application::SendRemoteAction;
use crate::domain::DeviceId;
use crate::domain::RemoteAction;
use crate::infrastructure::samsung::session::{
    ProbeObservation, Session, SessionError, SessionEvent,
};
use ::iced::window;
use std::sync::{Arc, Mutex};

pub struct SessionPackage(Arc<Mutex<Option<(SavedDevice, Session)>>>);

impl SessionPackage {
    pub fn new(device: SavedDevice, session: Session) -> Self {
        Self(Arc::new(Mutex::new(Some((device, session)))))
    }

    pub fn take(&self) -> Option<(SavedDevice, Session)> {
        self.0.lock().ok()?.take()
    }
}

impl Clone for SessionPackage {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl std::fmt::Debug for SessionPackage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionPackage([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairFlowError {
    Session(SessionError),
    Setup(SetupError),
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectFlowError {
    Session(SessionError),
    Reconnect(ReconnectError),
    CredentialSave,
    Cancelled,
}

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
        result: Result<ProbeObservation, SessionError>,
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
        event: SessionEvent,
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
