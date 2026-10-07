use crate::application::command::{RemoteActionOutcome, RemoteActionRejection, SendRemoteAction};
use crate::domain::DeviceId;

/// Application-owned control availability for the current snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlStatus {
    Unavailable(RemoteActionRejection),
}

#[derive(Debug, Clone, Copy, Default)]
enum Selection {
    #[default]
    None,
}

/// The application's control state snapshot. Device selection is introduced
/// by P1-M7; this milestone can represent only the initial, unselected state.
#[derive(Debug, Clone, Default)]
pub struct State {
    selection: Selection,
}

impl State {
    /// Creates a new state with no selected TV.
    pub const fn none() -> Self {
        Self {
            selection: Selection::None,
        }
    }

    /// Returns the currently selected device, if any.
    pub const fn selected_device(&self) -> Option<DeviceId> {
        match self.selection {
            Selection::None => None,
        }
    }

    pub const fn control_status(&self) -> ControlStatus {
        match self.selection {
            Selection::None => ControlStatus::Unavailable(RemoteActionRejection::NoSelectedTv),
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
}
