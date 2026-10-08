use sha2::{Digest, Sha256};

use crate::domain::DeviceId;

use super::tv_address::TvHost;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CertificatePin([u8; 32]);

impl CertificatePin {
    pub fn from_der(der: &[u8]) -> Self {
        Self(Sha256::digest(der).into())
    }

    pub fn from_hex(value: &str) -> Option<Self> {
        if value.len() != 64 {
            return None;
        }
        let mut bytes = [0u8; 32];
        for (index, chunk) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let high = (chunk[0] as char).to_digit(16)? as u8;
            let low = (chunk[1] as char).to_digit(16)? as u8;
            bytes[index] = (high << 4) | low;
        }
        Some(Self(bytes))
    }

    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}

impl std::fmt::Debug for CertificatePin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CertificatePin([redacted])")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustRecord {
    pub host: TvHost,
    pub pin: CertificatePin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustError {
    Unavailable,
    Corrupt,
}

pub trait CertificateTrustStore {
    fn load(&self, device: DeviceId) -> Result<Option<TrustRecord>, TrustError>;
    fn save(&self, device: DeviceId, record: &TrustRecord) -> Result<(), TrustError>;
    fn delete(&self, device: DeviceId) -> Result<(), TrustError>;
}
