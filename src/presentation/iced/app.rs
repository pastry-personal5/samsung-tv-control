use super::message::{Message, Shortcut};
use super::view;
use super::view_model::{rejection_message, ViewModel};
use crate::application::RemoteActionOutcome;
use crate::State;
use ::iced::keyboard::{self, key, Key, Modifiers};
use ::iced::{event, window, Element, Event, Size, Subscription, Task};

pub struct App {
    main_window: Option<window::Id>,
    settings_window: Option<window::Id>,
    view_model: ViewModel,
    app_state: State,
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
                _ => Task::none(),
            },
            Message::AttemptRemoteAction(request) => {
                match self.app_state.evaluate_remote_action(request) {
                    RemoteActionOutcome::Rejected { reason, .. } => self.publish(
                        super::view_model::MessageSeverity::Warning,
                        super::view_model::MessageSource::MainWindow,
                        format!("Remote action not sent. {}", rejection_message(reason)),
                    ),
                    RemoteActionOutcome::Eligible(_) => Task::none(),
                }
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
        modifiers,
        repeat: false,
        ..
    }) = event
    else {
        return None;
    };

    shortcut_for(&key, modifiers).map(|shortcut| Message::Shortcut { window, shortcut })
}

fn shortcut_for(key: &Key, modifiers: Modifiers) -> Option<Shortcut> {
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
        app.view_model.update_control_state(&app.app_state);

        let request = crate::SendRemoteAction::new(
            crate::DeviceId::new(42),
            generation,
            crate::RemoteAction::PowerToggle,
        );
        let _ = app.update(Message::AttemptRemoteAction(request));

        assert!(app.view_model.messages().entries().is_empty());
        assert!(app.view_model.control_state().remote_actions_enabled());
    }
}
