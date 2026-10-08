use std::fmt;

/// Opaque pairing credential. Debug output never contains its value.
pub struct PairingToken(String);

impl PairingToken {
    pub fn from_stored(value: String) -> Option<Self> {
        if !value.is_empty()
            && value.len() <= 1024
            && value.bytes().all(|byte| byte.is_ascii_graphic())
        {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for PairingToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PairingToken([redacted])")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretError {
    Unavailable,
    InvalidToken,
}

pub trait SecretStore {
    fn load(&self, device: crate::domain::DeviceId) -> Result<Option<PairingToken>, SecretError>;
    fn save(
        &self,
        device: crate::domain::DeviceId,
        token: &PairingToken,
    ) -> Result<(), SecretError>;
    fn delete(&self, device: crate::domain::DeviceId) -> Result<(), SecretError>;
}
