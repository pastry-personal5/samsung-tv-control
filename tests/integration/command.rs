use samsung_tv_remote::{
    application::{RemoteActionOutcome, RemoteActionRejection, SendRemoteAction},
    domain::{DeviceId, RemoteAction},
    State,
};

#[test]
fn send_remote_action_preserves_its_target_and_action() {
    let device_id = DeviceId::new(1);
    let action = RemoteAction::PowerToggle;
    let request = SendRemoteAction::new(device_id, action);

    assert_eq!(request.target(), device_id);
    assert_eq!(request.action(), action);
}

#[test]
fn requests_with_different_targets_are_not_equal() {
    let cmd1 = SendRemoteAction::new(DeviceId::new(1), RemoteAction::PowerToggle);
    let cmd2 = SendRemoteAction::new(DeviceId::new(2), RemoteAction::PowerToggle);

    assert_ne!(cmd1, cmd2);
}

#[test]
fn requests_with_different_actions_are_not_equal() {
    let cmd1 = SendRemoteAction::new(DeviceId::new(1), RemoteAction::PowerToggle);
    let cmd2 = SendRemoteAction::new(DeviceId::new(1), RemoteAction::Up);

    assert_ne!(cmd1, cmd2);
}

#[test]
fn send_remote_action_accepts_each_supported_semantic_action() {
    let device_id = DeviceId::new(1);

    for action in [
        RemoteAction::PowerToggle,
        RemoteAction::Up,
        RemoteAction::Down,
        RemoteAction::Left,
        RemoteAction::Right,
        RemoteAction::Select,
        RemoteAction::Back,
        RemoteAction::Home,
        RemoteAction::Mute,
        RemoteAction::VolumeUp,
        RemoteAction::VolumeDown,
    ] {
        let request = SendRemoteAction::new(device_id, action);
        assert_eq!(request.target(), device_id);
        assert_eq!(request.action(), action);
    }
}

#[test]
fn unselected_state_rejects_each_request_without_changing_its_target() {
    let state = State::none();

    for (index, action) in [
        RemoteAction::PowerToggle,
        RemoteAction::Up,
        RemoteAction::Down,
        RemoteAction::Left,
        RemoteAction::Right,
        RemoteAction::Select,
        RemoteAction::Back,
        RemoteAction::Home,
        RemoteAction::Mute,
        RemoteAction::VolumeUp,
        RemoteAction::VolumeDown,
    ]
    .into_iter()
    .enumerate()
    {
        let id = DeviceId::new(index as u64 + 1);
        let request = SendRemoteAction::new(id, action);

        assert_eq!(
            state.attempt_remote_action(request.clone()),
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NoSelectedTv,
            }
        );
        assert_eq!(state.selected_device(), None);
    }
}
