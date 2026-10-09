use samsung_tv_remote::domain::RemoteAction;

#[test]
fn remote_action_covers_every_rendered_button_intent() {
    let actions = [
        RemoteAction::PowerToggle,
        RemoteAction::Up,
        RemoteAction::Down,
        RemoteAction::Left,
        RemoteAction::Right,
        RemoteAction::Enter,
        RemoteAction::Back,
        RemoteAction::Home,
        RemoteAction::PlayPause,
        RemoteAction::Mute,
        RemoteAction::VolumeUp,
        RemoteAction::VolumeDown,
    ];

    assert_eq!(actions.len(), 12);
    assert_eq!(
        actions
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        12
    );
}

#[test]
fn equal_actions_compare_equal_and_distinct_actions_do_not() {
    assert_eq!(RemoteAction::PowerToggle, RemoteAction::PowerToggle);
    assert_ne!(RemoteAction::PowerToggle, RemoteAction::Up);
}

#[test]
fn remote_actions_can_be_copied_into_requests() {
    fn takes_copy<T: Copy>(_: T) {}
    takes_copy(RemoteAction::Home);
}

#[test]
fn remote_actions_can_be_found_in_a_hash_set() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(RemoteAction::Mute);
    assert!(set.contains(&RemoteAction::Mute));
}
