use super::view_model::PrimaryView;
use crate::application::SendRemoteAction;
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shortcut {
    Navigate(PrimaryView),
    OpenSettings,
    GrowMessages,
    ShrinkMessages,
}
