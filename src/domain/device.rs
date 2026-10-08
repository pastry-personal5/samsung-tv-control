use std::fmt;

/// Opaque local identity for a saved device record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeviceId(u128);

impl DeviceId {
    pub const fn new(id: u64) -> Self {
        Self(id as u128)
    }

    /// Generates a local record ID independently of TV names or network data.
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().as_u128())
    }

    pub fn parse_record_key(key: &str) -> Option<Self> {
        key.strip_prefix("dev_")?.parse::<u128>().ok().map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceDisplay {
    id: DeviceId,
    label: String,
}

impl DeviceDisplay {
    pub fn new(id: DeviceId, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    pub const fn id(&self) -> DeviceId {
        self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "dev_{}", self.0)
    }
}
