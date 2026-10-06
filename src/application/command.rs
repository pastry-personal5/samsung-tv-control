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
}
