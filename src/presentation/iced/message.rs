use super::view_model::PrimaryView;
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
}
