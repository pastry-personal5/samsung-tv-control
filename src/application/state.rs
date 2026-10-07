use crate::application::command::{RemoteActionOutcome, SendRemoteAction};
use crate::domain::DeviceId;

/// The application's control state snapshot.
/// 
/// Holds the optional selected device identifier.
#[derive(Debug, Clone, Default)]
pub struct State {
    selected_device: Option<DeviceId>,
}

impl State {
    /// Creates a new state with no selected TV.
    pub const fn none() -> Self {
        Self { selected_device: None }
    }

    /// Returns the currently selected device, if any.
    pub const fn selected_device(&self) -> Option<DeviceId> {
        self.selected_device
    }

    /// Attempts to send a remote action.
    /// 
    /// Returns `NoSelectedTv` if no device is selected.
    pub fn attempt_remote_action(
        &self,
        _request: SendRemoteAction,
    ) -> RemoteActionOutcome {
        if self.selected_device.is_none() {
            RemoteActionOutcome::NoSelectedTv
        } else {
            // Future milestones will add accepted/pending outcomes
            RemoteActionOutcome::NoSelectedTv
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
    }

    #[test]
    fn attempt_remote_action_returns_no_selected_tv_with_none_state() {
        let state = State::none();
        let request = SendRemoteAction::new(DeviceId::new(1), RemoteAction::PowerToggle);
        let outcome = state.attempt_remote_action(request);
        assert_eq!(outcome, RemoteActionOutcome::NoSelectedTv);
    }
}
