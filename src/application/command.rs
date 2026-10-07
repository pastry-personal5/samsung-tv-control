use crate::domain::{DeviceId, RemoteAction};

/// Typed request to dispatch a semantic remote action to one saved device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendRemoteAction {
    target: DeviceId,
    action: RemoteAction,
}

impl SendRemoteAction {
    pub const fn new(target: DeviceId, action: RemoteAction) -> Self {
        Self { target, action }
    }

    pub const fn target(&self) -> DeviceId {
        self.target
    }

    pub const fn action(&self) -> RemoteAction {
        self.action
    }
}

/// Result of attempting a remote action.
/// 
/// When no device is selected, all attempts return `NoSelectedTv`.
/// Future milestones will add accepted/pending outcomes and error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteActionOutcome {
    /// No TV is selected; the action cannot proceed.
    NoSelectedTv,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_preserves_target_and_action() {
        let device_id = DeviceId::new(1);
        let action = RemoteAction::PowerToggle;
        let request = SendRemoteAction::new(device_id, action);

        assert_eq!(request.target(), device_id);
        assert_eq!(request.action(), action);
    }

    #[test]
    fn outcome_is_no_selected_tv_when_no_device() {
        // The outcome variant exists and can be constructed/tested
        let outcome = RemoteActionOutcome::NoSelectedTv;
        assert_eq!(outcome, RemoteActionOutcome::NoSelectedTv);
    }
}
