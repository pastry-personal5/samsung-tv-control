use std::sync::OnceLock;

use ::iced::widget::image;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Icon {
    Power,
    Remote,
    Sources,
    Apps,
    TextInput,
    Settings,
    Wake,
    Retry,
    Cancel,
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Home,
    Mute,
    VolumeDown,
    VolumeUp,
}

const ARTWORK: [&[u8]; 19] = [
    include_bytes!("../../../assets/icons/power.png"),
    include_bytes!("../../../assets/icons/remote.png"),
    include_bytes!("../../../assets/icons/sources.png"),
    include_bytes!("../../../assets/icons/apps.png"),
    include_bytes!("../../../assets/icons/text_input.png"),
    include_bytes!("../../../assets/icons/settings.png"),
    include_bytes!("../../../assets/icons/wake.png"),
    include_bytes!("../../../assets/icons/retry.png"),
    include_bytes!("../../../assets/icons/cancel.png"),
    include_bytes!("../../../assets/icons/up.png"),
    include_bytes!("../../../assets/icons/down.png"),
    include_bytes!("../../../assets/icons/left.png"),
    include_bytes!("../../../assets/icons/right.png"),
    include_bytes!("../../../assets/icons/enter.png"),
    include_bytes!("../../../assets/icons/back.png"),
    include_bytes!("../../../assets/icons/home.png"),
    include_bytes!("../../../assets/icons/mute.png"),
    include_bytes!("../../../assets/icons/volume_down.png"),
    include_bytes!("../../../assets/icons/volume_up.png"),
];

static HANDLES: OnceLock<Vec<image::Handle>> = OnceLock::new();

impl Icon {
    pub fn image(self, size: f32, enabled: bool) -> image::Image {
        let handles = HANDLES.get_or_init(|| {
            ARTWORK
                .iter()
                .map(|bytes| image::Handle::from_bytes(*bytes))
                .collect()
        });
        image(handles[self as usize].clone())
            .width(size)
            .height(size)
            .opacity(if enabled { 1.0_f32 } else { 0.38_f32 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_is_an_embedded_png() {
        assert_eq!(ARTWORK.len(), 19);
        for bytes in ARTWORK {
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
            assert!(bytes.len() > 200);
            assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 96);
            assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 96);
        }
    }
}
