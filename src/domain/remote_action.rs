/// Finite semantic actions exposed by the Remote View.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoteAction {
    /// Toggle the TV power state.
    PowerToggle,
    Up,
    Down,
    Left,
    Right,
    Select,
    Back,
    Home,
    Mute,
    VolumeUp,
    VolumeDown,
}
