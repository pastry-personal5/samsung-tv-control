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
    /// A visual-remote intent resolved to Play or Pause by the application.
    PlayPause,
    /// Start playback on the TV.
    Play,
    /// Pause playback on the TV.
    Pause,
    Mute,
    VolumeUp,
    VolumeDown,
}

impl RemoteAction {
    pub const LIVE_ACTIONS: [Self; 11] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Enter,
        Self::Back,
        Self::Home,
        Self::PlayPause,
        Self::Mute,
        Self::VolumeUp,
        Self::VolumeDown,
    ];

    /// Actions that can be encoded as Samsung remote keys.
    pub const ENCODED_ACTIONS: [Self; 12] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Enter,
        Self::Back,
        Self::Home,
        Self::Play,
        Self::Pause,
        Self::Mute,
        Self::VolumeUp,
        Self::VolumeDown,
    ];
}
