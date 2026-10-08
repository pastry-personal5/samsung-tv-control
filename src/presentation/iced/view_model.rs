use std::collections::VecDeque;

use crate::application::{ConnectionState, ControlStatus, PairingState, RemoteActionRejection};
use crate::RemoteAction;
use crate::State;

pub const DEFAULT_MESSAGE_PANE_HEIGHT: u16 = 176;
pub const MIN_MESSAGE_PANE_HEIGHT: u16 = 120;
pub const MAX_MESSAGE_PANE_HEIGHT: u16 = 360;
pub const MESSAGE_FEED_CAPACITY: usize = 100;

pub(super) const fn remote_action_label(action: RemoteAction) -> &'static str {
    match action {
        RemoteAction::PowerToggle => "Power Toggle",
        RemoteAction::Up => "Up",
        RemoteAction::Down => "Down",
        RemoteAction::Left => "Left",
        RemoteAction::Right => "Right",
        RemoteAction::Select => "Enter",
        RemoteAction::Back => "Back",
        RemoteAction::Home => "Home",
        RemoteAction::Mute => "Mute",
        RemoteAction::VolumeUp => "Volume Up",
        RemoteAction::VolumeDown => "Volume Down",
    }
}

pub(super) fn feed_is_at_bottom(content_height: f32, viewport_height: f32, offset: f32) -> bool {
    (content_height - viewport_height).max(0.0) - offset <= 1.0
}

pub(super) const fn rejection_message(reason: RemoteActionRejection) -> &'static str {
    match reason {
        RemoteActionRejection::NoSelectedTv => "No TV selected. Open Settings to choose a TV.",
        RemoteActionRejection::WrongTarget | RemoteActionRejection::StaleSelectionGeneration => {
            "TV selection changed. Use the current TV before sending remote actions."
        }
        RemoteActionRejection::PairingRequired => "Pair this TV before sending remote actions.",
        RemoteActionRejection::NotConnected => "Connect to this TV before sending remote actions.",
        RemoteActionRejection::DeferredAction => "Power control is planned for a later milestone.",
        RemoteActionRejection::UnverifiedAction => {
            "This action has not been verified on the selected TV."
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifecycleStatusText {
    pub label: &'static str,
    pub guidance: &'static str,
}

const fn pairing_status_text(state: PairingState) -> LifecycleStatusText {
    match state {
        PairingState::NotStarted => LifecycleStatusText {
            label: "Not paired",
            guidance: "Pair this TV before sending remote actions.",
        },
        PairingState::InProgress => LifecycleStatusText {
            label: "Pairing in progress",
            guidance: "Wait for pairing to finish before sending remote actions.",
        },
        PairingState::Ready => LifecycleStatusText {
            label: "Paired",
            guidance: "Pairing is ready for this TV.",
        },
        PairingState::Failed => LifecycleStatusText {
            label: "Pairing needs attention",
            guidance: "Try pairing this TV again.",
        },
    }
}

const fn connection_status_text(state: ConnectionState) -> LifecycleStatusText {
    match state {
        ConnectionState::NotConnected => LifecycleStatusText {
            label: "Not connected",
            guidance: "Connect to this TV before sending remote actions.",
        },
        ConnectionState::Connecting => LifecycleStatusText {
            label: "Connecting",
            guidance: "Wait for the connection to finish before sending remote actions.",
        },
        ConnectionState::Ready => LifecycleStatusText {
            label: "Connected",
            guidance: "Connection is ready for this TV.",
        },
        ConnectionState::Failed => LifecycleStatusText {
            label: "Connection needs attention",
            guidance: "Try connecting to this TV again.",
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimaryView {
    Remote,
    Sources,
    Apps,
    TextInput,
}

impl PrimaryView {
    pub const fn title(self) -> &'static str {
        match self {
            Self::Remote => "Remote",
            Self::Sources => "Sources",
            Self::Apps => "Apps",
            Self::TextInput => "Text Input",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSeverity {
    Information,
    Warning,
}

impl MessageSeverity {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Information => "Info",
            Self::Warning => "Warning",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageSource {
    MainWindow,
    SettingsWindow,
}

impl MessageSource {
    pub const fn label(self) -> &'static str {
        match self {
            Self::MainWindow => "Main window",
            Self::SettingsWindow => "Settings",
        }
    }
}

/// A user-visible message already checked by the presentation boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayMessage {
    pub sequence: u64,
    pub severity: MessageSeverity,
    pub source: MessageSource,
    pub text: String,
}

#[derive(Debug)]
pub struct MessageFeed {
    entries: VecDeque<DisplayMessage>,
    next_sequence: u64,
    capacity: usize,
    at_bottom: bool,
    unread_count: usize,
}

impl MessageFeed {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            next_sequence: 1,
            capacity,
            at_bottom: true,
            unread_count: 0,
        }
    }

    pub fn entries(&self) -> &VecDeque<DisplayMessage> {
        &self.entries
    }

    pub fn at_bottom(&self) -> bool {
        self.at_bottom
    }

    pub fn unread_count(&self) -> usize {
        self.unread_count
    }

    pub fn append(
        &mut self,
        severity: MessageSeverity,
        source: MessageSource,
        text: impl Into<String>,
    ) -> bool {
        let follow = self.at_bottom;
        let entry = DisplayMessage {
            sequence: self.next_sequence,
            severity,
            source,
            text: text.into(),
        };
        self.next_sequence = self.next_sequence.saturating_add(1);

        if self.capacity == 0 {
            return false;
        }

        if self.entries.len() == self.capacity {
            let _ = self.entries.pop_front();
        }
        self.entries.push_back(entry);

        if follow {
            self.unread_count = 0;
        } else {
            self.unread_count = self.unread_count.saturating_add(1);
        }

        follow
    }

    pub fn set_at_bottom(&mut self, at_bottom: bool) {
        self.at_bottom = at_bottom;
        if at_bottom {
            self.unread_count = 0;
        }
    }
}

/// Projection of the application's control state into the presentation layer.
///
/// This exposes the no-selected-TV fact and the reason for disabled controls
/// without exposing Iced or application implementation details.
#[derive(Debug, Clone)]
pub struct ControlState {
    /// Safe display information for the selected device, if any.
    pub selected_device: Option<crate::DeviceDisplay>,
    /// The application-owned generation associated with the projection.
    pub selection_generation: u64,
    /// Human-readable explanation for why controls are unavailable.
    pub disabled_reason: Option<&'static str>,
    /// Pairing lifecycle text derived from application state.
    pub pairing: LifecycleStatusText,
    /// Connection lifecycle text derived from application state.
    pub connection: LifecycleStatusText,
    action_statuses: Vec<(RemoteAction, Option<&'static str>)>,
}

impl ControlState {
    /// Creates a control state projection from the application state.
    pub fn from_application_state(app_state: &State) -> Self {
        let selected_device = app_state.selected_device_display().cloned();
        let selection_generation = app_state.selection_generation();
        let pairing = pairing_status_text(app_state.pairing_state());
        let connection = connection_status_text(app_state.connection_state());

        let action_statuses = RemoteAction::LIVE_ACTIONS
            .into_iter()
            .chain([RemoteAction::PowerToggle])
            .map(|action| {
                let reason = match app_state.control_status(action) {
                    ControlStatus::Available => None,
                    ControlStatus::Unavailable(reason) => Some(rejection_message(reason)),
                };
                (action, reason)
            })
            .collect::<Vec<_>>();
        let disabled_reason = action_statuses
            .iter()
            .find(|(action, _)| *action == RemoteAction::Up)
            .and_then(|(_, reason)| *reason);
        Self {
            selected_device,
            selection_generation,
            disabled_reason,
            pairing,
            connection,
            action_statuses,
        }
    }

    /// Returns true if no TV is selected.
    pub fn is_no_tv_selected(&self) -> bool {
        self.selected_device.is_none()
    }

    pub fn remote_actions_enabled(&self) -> bool {
        RemoteAction::LIVE_ACTIONS
            .into_iter()
            .any(|action| self.action_disabled_reason(action).is_none())
    }

    pub fn action_disabled_reason(&self, action: RemoteAction) -> Option<&'static str> {
        self.action_statuses
            .iter()
            .find(|(candidate, _)| *candidate == action)
            .and_then(|(_, reason)| *reason)
    }
}

#[derive(Debug)]
pub struct ViewModel {
    primary_view: PrimaryView,
    message_pane_height: u16,
    messages: MessageFeed,
    control_state: ControlState,
    activity: VecDeque<String>,
}

impl Default for ViewModel {
    fn default() -> Self {
        Self::new(&State::none())
    }
}

impl ViewModel {
    pub fn new(app_state: &State) -> Self {
        Self {
            primary_view: PrimaryView::Remote,
            message_pane_height: DEFAULT_MESSAGE_PANE_HEIGHT,
            messages: MessageFeed::new(MESSAGE_FEED_CAPACITY),
            control_state: ControlState::from_application_state(app_state),
            activity: VecDeque::new(),
        }
    }
    pub fn primary_view(&self) -> PrimaryView {
        self.primary_view
    }

    pub fn select_view(&mut self, primary_view: PrimaryView) {
        self.primary_view = primary_view;
    }

    pub fn message_pane_height(&self) -> u16 {
        self.message_pane_height
    }

    pub fn resize_message_pane(&mut self, height: u16) {
        self.message_pane_height = height.clamp(MIN_MESSAGE_PANE_HEIGHT, MAX_MESSAGE_PANE_HEIGHT);
    }

    pub fn messages(&self) -> &MessageFeed {
        &self.messages
    }

    pub fn messages_mut(&mut self) -> &mut MessageFeed {
        &mut self.messages
    }

    /// Returns the control state projection.
    pub fn control_state(&self) -> &ControlState {
        &self.control_state
    }

    /// Updates the control state from the application state.
    pub fn update_control_state(&mut self, app_state: &State) {
        self.control_state = ControlState::from_application_state(app_state);
    }

    pub fn activity(&self) -> &VecDeque<String> {
        &self.activity
    }

    pub fn add_activity(&mut self, text: impl Into<String>) {
        if self.activity.len() >= MESSAGE_FEED_CAPACITY {
            let _ = self.activity.pop_front();
        }
        self.activity.push_back(text.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remote_action_labels_use_canonical_control_names() {
        let expected = [
            (RemoteAction::PowerToggle, "Power Toggle"),
            (RemoteAction::Up, "Up"),
            (RemoteAction::Down, "Down"),
            (RemoteAction::Left, "Left"),
            (RemoteAction::Right, "Right"),
            (RemoteAction::Select, "Enter"),
            (RemoteAction::Back, "Back"),
            (RemoteAction::Home, "Home"),
            (RemoteAction::Mute, "Mute"),
            (RemoteAction::VolumeUp, "Volume Up"),
            (RemoteAction::VolumeDown, "Volume Down"),
        ];
        for (action, label) in expected {
            assert_eq!(remote_action_label(action), label);
        }
    }

    #[test]
    fn new_state_has_no_selected_tv() {
        let app_state = State::none();
        let control_state = ControlState::from_application_state(&app_state);

        assert_eq!(control_state.selected_device, None);
        assert!(control_state.is_no_tv_selected());
    }

    #[test]
    fn control_state_provides_disabled_reason_for_no_tv() {
        let app_state = State::none();
        let control_state = ControlState::from_application_state(&app_state);

        assert_eq!(
            control_state.disabled_reason,
            Some(rejection_message(RemoteActionRejection::NoSelectedTv))
        );
    }

    #[test]
    fn control_state_projects_safe_selected_device_and_generation() {
        let mut app_state = State::none();
        let _ = app_state.select_device(crate::DeviceDisplay::new(
            crate::DeviceId::new(9),
            "Studio TV",
        ));

        let control_state = ControlState::from_application_state(&app_state);

        assert_eq!(
            control_state
                .selected_device
                .as_ref()
                .map(|device| device.label()),
            Some("Studio TV")
        );
        assert_eq!(control_state.selection_generation, 1);
        assert_eq!(
            control_state.disabled_reason,
            Some(rejection_message(RemoteActionRejection::PairingRequired))
        );
    }

    #[test]
    fn control_state_projects_distinct_lifecycle_labels_and_safe_guidance() {
        let mut app_state = State::none();
        let _ = app_state.select_device(crate::DeviceDisplay::new(
            crate::DeviceId::new(9),
            "Studio TV",
        ));
        let generation = app_state.selection_generation();
        let _ = app_state.set_pairing_state(generation, PairingState::Failed);
        let _ = app_state.set_connection_state(generation, ConnectionState::Connecting);

        let control_state = ControlState::from_application_state(&app_state);

        assert_eq!(control_state.pairing.label, "Pairing needs attention");
        assert_eq!(control_state.pairing.guidance, "Try pairing this TV again.");
        assert_eq!(control_state.connection.label, "Connecting");
        assert_eq!(
            control_state.connection.guidance,
            "Wait for the connection to finish before sending remote actions."
        );
    }

    #[test]
    fn control_state_enables_only_when_the_application_policy_is_eligible() {
        let mut app_state = State::none();
        let _ = app_state.select_device(crate::DeviceDisplay::new(
            crate::DeviceId::new(9),
            "Studio TV",
        ));
        let generation = app_state.selection_generation();

        assert!(!ControlState::from_application_state(&app_state).remote_actions_enabled());
        let _ = app_state.set_pairing_state(generation, PairingState::Ready);
        assert!(!ControlState::from_application_state(&app_state).remote_actions_enabled());
        let _ = app_state.set_connection_state(generation, ConnectionState::Ready);
        let _ = app_state.set_verified_actions(generation, [RemoteAction::Up]);

        let control_state = ControlState::from_application_state(&app_state);
        assert!(control_state.remote_actions_enabled());
        assert_eq!(control_state.disabled_reason, None);
        assert_eq!(
            control_state.action_disabled_reason(RemoteAction::PowerToggle),
            Some("Power control is planned for a later milestone.")
        );
    }

    #[test]
    fn route_selection_changes_only_the_primary_view() {
        let mut view_model = ViewModel::default();
        let _ = view_model.messages_mut().append(
            MessageSeverity::Information,
            MessageSource::MainWindow,
            "Keep this message",
        );

        for route in [
            PrimaryView::Sources,
            PrimaryView::Apps,
            PrimaryView::TextInput,
            PrimaryView::Remote,
        ] {
            view_model.select_view(route);
            assert_eq!(view_model.primary_view(), route);
            assert_eq!(
                view_model.message_pane_height(),
                DEFAULT_MESSAGE_PANE_HEIGHT
            );
            assert_eq!(view_model.messages().entries().len(), 1);
            assert!(view_model.control_state().is_no_tv_selected());
        }
    }

    #[test]
    fn message_feed_evicts_oldest_entry_and_keeps_sequence_order() {
        let mut feed = MessageFeed::new(2);
        let _ = feed.append(
            MessageSeverity::Information,
            MessageSource::MainWindow,
            "First",
        );
        let _ = feed.append(
            MessageSeverity::Warning,
            MessageSource::SettingsWindow,
            "Second",
        );
        let _ = feed.append(MessageSeverity::Warning, MessageSource::MainWindow, "Third");

        let entries: Vec<_> = feed.entries().iter().collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].sequence, 2);
        assert_eq!(entries[0].text, "Second");
        assert_eq!(entries[1].sequence, 3);
        assert_eq!(entries[1].text, "Third");
    }

    #[test]
    fn message_feed_follows_only_while_at_bottom() {
        let mut feed = MessageFeed::new(3);
        assert!(feed.append(
            MessageSeverity::Information,
            MessageSource::MainWindow,
            "Following",
        ));
        assert_eq!(feed.unread_count(), 0);

        feed.set_at_bottom(false);
        assert!(!feed.append(
            MessageSeverity::Information,
            MessageSource::MainWindow,
            "Do not move the reader",
        ));
        assert_eq!(feed.unread_count(), 1);

        feed.set_at_bottom(true);
        assert_eq!(feed.unread_count(), 0);
    }

    #[test]
    fn feed_bottom_detection_uses_pixel_distance() {
        assert!(feed_is_at_bottom(100.0, 200.0, 0.0));
        assert!(feed_is_at_bottom(10_000.0, 100.0, 9_899.5));
        assert!(!feed_is_at_bottom(10_000.0, 100.0, 9_890.0));
    }

    #[test]
    fn message_pane_height_stays_within_usable_bounds() {
        let mut view_model = ViewModel::default();
        view_model.resize_message_pane(0);
        assert_eq!(view_model.message_pane_height(), MIN_MESSAGE_PANE_HEIGHT);

        view_model.resize_message_pane(u16::MAX);
        assert_eq!(view_model.message_pane_height(), MAX_MESSAGE_PANE_HEIGHT);
    }
}
