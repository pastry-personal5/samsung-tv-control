use crate::domain::{DeviceId, RemoteAction};

/// Typed request to dispatch a semantic remote action to one saved device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendRemoteAction {
    target: DeviceId,
    selection_generation: u64,
    action: RemoteAction,
}

impl SendRemoteAction {
    pub const fn new(target: DeviceId, selection_generation: u64, action: RemoteAction) -> Self {
        Self {
            target,
            selection_generation,
            action,
        }
    }

    pub const fn target(&self) -> DeviceId {
        self.target
    }

    /// Selection epoch observed when the user formed this request.
    pub const fn selection_generation(&self) -> u64 {
        self.selection_generation
    }

    pub const fn action(&self) -> RemoteAction {
        self.action
    }
}

/// A reason why a remote action cannot be admitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteActionRejection {
    NoSelectedTv,
    WrongTarget,
    StaleSelectionGeneration,
    PairingRequired,
    NotConnected,
}

/// Result of evaluating a typed remote action against application state.
/// Rejection retains the original request so its target is never replaced by
/// a presentation-layer placeholder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteActionOutcome {
    Rejected {
        request: SendRemoteAction,
        reason: RemoteActionRejection,
    },
    /// All local policy preconditions passed. This is not a send result.
    Eligible(SendRemoteAction),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_preserves_target_and_action() {
        let device_id = DeviceId::new(1);
        let action = RemoteAction::PowerToggle;
        let request = SendRemoteAction::new(device_id, 3, action);

        assert_eq!(request.target(), device_id);
        assert_eq!(request.selection_generation(), 3);
        assert_eq!(request.action(), action);
    }

    #[test]
    fn rejected_outcome_preserves_request_and_reason() {
        let request = SendRemoteAction::new(DeviceId::new(7), 2, RemoteAction::Enter);
        let outcome = RemoteActionOutcome::Rejected {
            request: request.clone(),
            reason: RemoteActionRejection::NoSelectedTv,
        };

        assert_eq!(
            outcome,
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NoSelectedTv,
            }
        );
    }
}
