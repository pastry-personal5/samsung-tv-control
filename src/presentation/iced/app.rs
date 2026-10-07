use super::message::Message;
use super::view;
use super::view_model::ViewModel;
use crate::State;
use ::iced::{Element, Size, Subscription, Task, window};

pub struct App {
    main_window: Option<window::Id>,
    settings_window: Option<window::Id>,
    view_model: ViewModel,
    app_state: State,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let app_state = State::none();
        let view_model = ViewModel::default();
        (
            Self {
                main_window: None,
                settings_window: None,
                view_model,
                app_state,
            },
            Task::done(Message::OpenMainWindow),
        )
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
            Message::MainWindowOpened(id) => {
                self.main_window = Some(id);
                Task::none()
            }
            Message::SettingsWindowOpened(id) => {
                self.settings_window = Some(id);
                Task::none()
            }
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
            Message::AttemptRemoteAction => {
                // Attempt the remote action against the application state.
                // This returns a typed result (e.g., NoSelectedTv).
                // For now, we just evaluate and render the result without
                // sending any actual command to a transport.
                let outcome = self.app_state.attempt_remote_action(
                    crate::SendRemoteAction::new(
                        crate::DeviceId::new(0),
                        crate::RemoteAction::PowerToggle,
                    ),
                );
                self.view_model
                    .messages_mut()
                    .append(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::MainWindow,
                        format!("Remote action result: {:?}", outcome),
                    );
                Task::none()
            }
            Message::RemoteActionResult(outcome) => {
                // Handle the result of a remote action attempt.
                // This is a placeholder for future async dispatch paths.
                self.view_model
                    .messages_mut()
                    .append(
                        super::view_model::MessageSeverity::Information,
                        super::view_model::MessageSource::MainWindow,
                        format!("Remote action result: {:?}", outcome),
                    );
                Task::none()
            }
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.settings_window == Some(window) {
            view::settings_window()
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
        window::close_events().map(Message::WindowClosed)
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

pub fn run() -> ::iced::Result {
    ::iced::daemon(App::new, App::update, App::view)
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
}
