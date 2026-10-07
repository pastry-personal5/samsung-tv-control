use super::view_model::PrimaryView;
use crate::application::command::RemoteActionOutcome;
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
    FeedScrolled { at_bottom: bool },
    /// Remote action intent to send to the application layer.
    /// When the user attempts to use a remote control, this message
    /// is dispatched to the presentation reducer which evaluates
    /// the request against the application state.
    AttemptRemoteAction,
    /// Result of a remote action attempt from the application layer.
    RemoteActionResult(RemoteActionOutcome),
}
