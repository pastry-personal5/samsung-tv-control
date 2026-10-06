use samsung_tv_remote::domain::RemoteAction;

#[test]
fn remote_action_covers_every_rendered_button_intent() {
    let actions = [
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
    ];

    assert_eq!(actions.len(), 11);
    assert_eq!(
        actions
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        11
    );
}

#[test]
fn remote_action_equality() {
    assert_eq!(RemoteAction::PowerToggle, RemoteAction::PowerToggle);
    assert_ne!(RemoteAction::PowerToggle, RemoteAction::Up);
}

#[test]
fn remote_action_is_copy() {
    fn takes_copy<T: Copy>(_: T) {}
    takes_copy(RemoteAction::Home);
}

#[test]
fn remote_action_is_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(RemoteAction::Mute);
    assert!(set.contains(&RemoteAction::Mute));
}
