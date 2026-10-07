use std::collections::VecDeque;

use crate::State;

pub const DEFAULT_MESSAGE_PANE_HEIGHT: u16 = 176;
pub const MIN_MESSAGE_PANE_HEIGHT: u16 = 120;
pub const MAX_MESSAGE_PANE_HEIGHT: u16 = 360;
pub const MESSAGE_FEED_CAPACITY: usize = 100;

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
            return follow;
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
    /// The selected device identifier, if any.
    pub selected_device: Option<crate::DeviceId>,
    /// Human-readable explanation for why controls are unavailable.
    pub disabled_reason: &'static str,
}

impl ControlState {
    /// Creates a control state projection from the application state.
    pub fn from_application_state(app_state: &State) -> Self {
        match app_state.selected_device() {
            Some(device_id) => {
                // Future milestones will provide the actual reason when device is selected
                Self {
                    selected_device: Some(device_id),
                    disabled_reason: "",
                }
            }
            None => Self {
                selected_device: None,
                disabled_reason: "No TV selected. Open Settings to choose a TV.",
            },
        }
    }

    /// Returns true if no TV is selected.
    pub fn is_no_tv_selected(&self) -> bool {
        self.selected_device.is_none()
    }
}

#[derive(Debug)]
pub struct ViewModel {
    primary_view: PrimaryView,
    message_pane_height: u16,
    messages: MessageFeed,
    control_state: ControlState,
}

impl Default for ViewModel {
    fn default() -> Self {
        Self {
            primary_view: PrimaryView::Remote,
            message_pane_height: DEFAULT_MESSAGE_PANE_HEIGHT,
            messages: MessageFeed::new(MESSAGE_FEED_CAPACITY),
            control_state: ControlState::from_application_state(&State::none()),
        }
    }
}

impl ViewModel {
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
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert!(!control_state.disabled_reason.is_empty());
    }

    #[test]
    fn route_selection_changes_only_the_primary_view() {
        let mut view_model = ViewModel::default();

        view_model.select_view(PrimaryView::Sources);
        assert_eq!(view_model.primary_view(), PrimaryView::Sources);
        assert_eq!(
            view_model.message_pane_height(),
            DEFAULT_MESSAGE_PANE_HEIGHT
        );
        assert!(view_model.messages().entries().is_empty());
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
    fn message_pane_height_stays_within_usable_bounds() {
        let mut view_model = ViewModel::default();
        view_model.resize_message_pane(0);
        assert_eq!(view_model.message_pane_height(), MIN_MESSAGE_PANE_HEIGHT);

        view_model.resize_message_pane(u16::MAX);
        assert_eq!(view_model.message_pane_height(), MAX_MESSAGE_PANE_HEIGHT);
    }
}
