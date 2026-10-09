use crate::domain::{DeviceDisplay, DeviceId};

use super::tv_address::TvHost;
use super::wake::WakeConfiguration;

/// Non-secret saved TV information. Credentials and certificate trust have
/// separate stores, both keyed by this generated local record ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavedDevice {
    pub id: DeviceId,
    pub label: String,
    pub host: TvHost,
    pub wake: WakeConfiguration,
}

impl SavedDevice {
    pub fn display(&self) -> DeviceDisplay {
        DeviceDisplay::new(self.id, self.label.clone())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepositoryError {
    Unavailable,
    Corrupt,
}

pub trait DeviceRepository {
    fn list(&self) -> Result<Vec<SavedDevice>, RepositoryError>;
    fn save(&self, device: &SavedDevice) -> Result<(), RepositoryError>;
    fn delete(&self, id: DeviceId) -> Result<(), RepositoryError>;
    fn selected(&self) -> Result<Option<DeviceId>, RepositoryError>;
    fn select(&self, id: Option<DeviceId>) -> Result<(), RepositoryError>;
}
