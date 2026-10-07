use crate::application::command::{RemoteActionOutcome, RemoteActionRejection, SendRemoteAction};
use crate::domain::{DeviceDisplay, DeviceId};

/// Application-owned control availability for the current snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlStatus {
    Unavailable(RemoteActionRejection),
}

#[derive(Debug, Clone, Default)]
enum Selection {
    #[default]
    None,
    Selected(DeviceDisplay),
}

/// The application's control state snapshot. Device selection is introduced
/// by P1-M7; this milestone can represent only the initial, unselected state.
#[derive(Debug, Clone, Default)]
pub struct State {
    selection: Selection,
    selection_generation: u64,
}

impl State {
    /// Creates a new state with no selected TV.
    pub const fn none() -> Self {
        Self {
            selection: Selection::None,
            selection_generation: 0,
        }
    }

    /// Returns the currently selected device, if any.
    pub const fn selected_device(&self) -> Option<DeviceId> {
        match &self.selection {
            Selection::None => None,
            Selection::Selected(device) => Some(device.id()),
        }
    }

    /// Returns safe display information for the selected device, if any.
    pub fn selected_device_display(&self) -> Option<&DeviceDisplay> {
        match &self.selection {
            Selection::None => None,
            Selection::Selected(device) => Some(device),
        }
    }

    /// Monotonically identifies the current selection epoch.
    pub const fn selection_generation(&self) -> u64 {
        self.selection_generation
    }

    /// Selects a caller-supplied known device for this session.
    ///
    /// Re-selecting the current identity is an idempotent no-op, including its
    /// display projection. A different identity starts a new selection epoch.
    pub fn select_device(&mut self, device: DeviceDisplay) -> bool {
        if self.selected_device() == Some(device.id()) {
            return false;
        }

        self.selection = Selection::Selected(device);
        self.selection_generation = self.selection_generation.saturating_add(1);
        true
    }

    /// Clears the session selection and starts a new epoch when one existed.
    pub fn clear_selection(&mut self) -> bool {
        if matches!(self.selection, Selection::None) {
            return false;
        }

        self.selection = Selection::None;
        self.selection_generation = self.selection_generation.saturating_add(1);
        true
    }

    pub const fn control_status(&self) -> ControlStatus {
        match &self.selection {
            Selection::None => ControlStatus::Unavailable(RemoteActionRejection::NoSelectedTv),
            Selection::Selected(_) => {
                ControlStatus::Unavailable(RemoteActionRejection::SelectedTvNotReady)
            }
        }
    }

    /// Attempts to send a remote action.
    ///
    /// Returns a rejection before any command can be admitted or sent.
    pub fn attempt_remote_action(&self, request: SendRemoteAction) -> RemoteActionOutcome {
        match self.control_status() {
            ControlStatus::Unavailable(reason) => RemoteActionOutcome::Rejected { request, reason },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RemoteAction;

    #[test]
    fn new_state_has_no_selected_tv() {
        let state = State::none();
        assert_eq!(state.selected_device(), None);
        assert_eq!(State::default().selected_device(), None);
        assert_eq!(
            state.control_status(),
            ControlStatus::Unavailable(RemoteActionRejection::NoSelectedTv)
        );
    }

    #[test]
    fn attempt_remote_action_rejects_and_preserves_the_requested_target() {
        let state = State::none();
        let request = SendRemoteAction::new(DeviceId::new(1), RemoteAction::PowerToggle);
        let outcome = state.attempt_remote_action(request.clone());
        assert_eq!(
            outcome,
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NoSelectedTv,
            }
        );
    }

    #[test]
    fn selection_and_clear_advance_the_generation_only_when_identity_changes() {
        let mut state = State::none();
        let living_room = DeviceDisplay::new(DeviceId::new(1), "Living Room");
        let renamed_living_room = DeviceDisplay::new(DeviceId::new(1), "TV");
        let bedroom = DeviceDisplay::new(DeviceId::new(2), "Bedroom");

        assert_eq!(state.selection_generation(), 0);
        assert!(state.select_device(living_room));
        assert_eq!(state.selected_device(), Some(DeviceId::new(1)));
        assert_eq!(
            state.selected_device_display().map(DeviceDisplay::label),
            Some("Living Room")
        );
        assert_eq!(state.selection_generation(), 1);

        assert!(!state.select_device(renamed_living_room));
        assert_eq!(
            state.selected_device_display().map(DeviceDisplay::label),
            Some("Living Room")
        );
        assert_eq!(state.selection_generation(), 1);

        assert!(state.select_device(bedroom));
        assert_eq!(state.selected_device(), Some(DeviceId::new(2)));
        assert_eq!(state.selection_generation(), 2);

        assert!(state.clear_selection());
        assert_eq!(state.selected_device(), None);
        assert_eq!(state.selected_device_display(), None);
        assert_eq!(state.selection_generation(), 3);
        assert!(!state.clear_selection());
        assert_eq!(state.selection_generation(), 3);
    }

    #[test]
    fn selected_device_stays_disabled_until_lifecycle_state_exists() {
        let mut state = State::none();
        let _ = state.select_device(DeviceDisplay::new(DeviceId::new(1), "Living Room"));
        let request = SendRemoteAction::new(DeviceId::new(1), RemoteAction::PowerToggle);

        assert_eq!(
            state.attempt_remote_action(request.clone()),
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::SelectedTvNotReady,
            }
        );
    }
}
