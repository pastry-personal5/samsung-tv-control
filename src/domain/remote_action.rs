/// Finite semantic actions exposed by the Remote View.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum RemoteAction {
    /// Toggle the TV power state.
    PowerToggle,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Home,
    Mute,
    VolumeUp,
    VolumeDown,
}

impl RemoteAction {
    pub const LIVE_ACTIONS: [Self; 10] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Enter,
        Self::Back,
        Self::Home,
        Self::Mute,
        Self::VolumeUp,
        Self::VolumeDown,
    ];
}
