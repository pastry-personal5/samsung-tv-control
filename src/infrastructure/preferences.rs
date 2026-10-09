use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::application::certificate_trust::{
    CertificatePin, CertificateTrustStore, TrustError, TrustRecord,
};
use crate::application::device_repository::{DeviceRepository, RepositoryError, SavedDevice};
use crate::application::tv_address::TvHost;
use crate::application::wake::{WakeConfiguration, WakeInterface};
use crate::domain::DeviceId;

const MAX_FILE_BYTES: u64 = 1024 * 1024;
const DEVICE_VERSION: u32 = 2;
const TRUST_VERSION: u32 = 1;

pub struct LocalDeviceRepository {
    file: JsonFile,
}

pub struct LocalCertificateTrustStore {
    file: JsonFile,
}

struct JsonFile {
    path: PathBuf,
    lock: Mutex<()>,
}

#[derive(Serialize, Deserialize)]
struct DiskDevices {
    version: u32,
    selected: Option<String>,
    devices: Vec<DiskDevice>,
}

#[derive(Serialize, Deserialize)]
struct DiskDevice {
    id: String,
    label: String,
    host: String,
    #[serde(default)]
    wake: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct DiskWake {
    wired: Option<String>,
    wifi: Option<String>,
    active: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct DiskTrust {
    version: u32,
    records: HashMap<String, DiskTrustRecord>,
}

#[derive(Serialize, Deserialize)]
struct DiskTrustRecord {
    host: String,
    certificate_sha256: String,
}

impl Default for DiskDevices {
    fn default() -> Self {
        Self {
            version: DEVICE_VERSION,
            selected: None,
            devices: Vec::new(),
        }
    }
}

impl Default for DiskTrust {
    fn default() -> Self {
        Self {
            version: TRUST_VERSION,
            records: HashMap::new(),
        }
    }
}

impl LocalDeviceRepository {
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            file: JsonFile::new(app_data_dir.join("devices.json")),
        }
    }
}

impl LocalCertificateTrustStore {
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            file: JsonFile::new(app_data_dir.join("trust.json")),
        }
    }
}

pub fn default_app_data_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join("Library/Application Support/Samsung TV Remote"))
}

impl DeviceRepository for LocalDeviceRepository {
    fn list(&self) -> Result<Vec<SavedDevice>, RepositoryError> {
        let _guard = self
            .file
            .lock
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?;
        let disk = self.load()?;
        disk.devices.into_iter().map(convert_device).collect()
    }

    fn save(&self, device: &SavedDevice) -> Result<(), RepositoryError> {
        let _guard = self
            .file
            .lock
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?;
        let mut disk = self.load()?;
        disk.version = DEVICE_VERSION;
        let record = DiskDevice {
            id: device.id.to_string(),
            label: device.label.clone(),
            host: device.host.as_str().to_owned(),
            wake: Some(
                serde_json::to_value(DiskWake {
                    wired: device.wake.wired.map(|mac| mac.to_string()),
                    wifi: device.wake.wifi.map(|mac| mac.to_string()),
                    active: device.wake.active.map(|active| match active {
                        WakeInterface::Wired => "wired".to_owned(),
                        WakeInterface::WiFi => "wifi".to_owned(),
                    }),
                })
                .map_err(|_| RepositoryError::Unavailable)?,
            ),
        };
        if let Some(existing) = disk.devices.iter_mut().find(|item| item.id == record.id) {
            *existing = record;
        } else {
            disk.devices.push(record);
        }
        self.file
            .write(&disk)
            .map_err(|_| RepositoryError::Unavailable)
    }

    fn delete(&self, id: DeviceId) -> Result<(), RepositoryError> {
        let _guard = self
            .file
            .lock
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?;
        let mut disk = self.load()?;
        disk.version = DEVICE_VERSION;
        disk.devices.retain(|item| item.id != id.to_string());
        if disk.selected.as_deref() == Some(id.to_string().as_str()) {
            disk.selected = None;
        }
        self.file
            .write(&disk)
            .map_err(|_| RepositoryError::Unavailable)
    }

    fn selected(&self) -> Result<Option<DeviceId>, RepositoryError> {
        let _guard = self
            .file
            .lock
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?;
        let disk = self.load()?;
        disk.selected
            .map(|key| DeviceId::parse_record_key(&key).ok_or(RepositoryError::Corrupt))
            .transpose()
    }

    fn select(&self, id: Option<DeviceId>) -> Result<(), RepositoryError> {
        let _guard = self
            .file
            .lock
            .lock()
            .map_err(|_| RepositoryError::Unavailable)?;
        let mut disk = self.load()?;
        disk.version = DEVICE_VERSION;
        if let Some(id) = id {
            if !disk.devices.iter().any(|item| item.id == id.to_string()) {
                return Err(RepositoryError::Corrupt);
            }
        }
        disk.selected = id.map(|id| id.to_string());
        self.file
            .write(&disk)
            .map_err(|_| RepositoryError::Unavailable)
    }
}

impl LocalDeviceRepository {
    fn load(&self) -> Result<DiskDevices, RepositoryError> {
        let disk: DiskDevices = self
            .file
            .read()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::InvalidData => RepositoryError::Corrupt,
                _ => RepositoryError::Unavailable,
            })?
            .unwrap_or_default();
        if disk.version != 1 && disk.version != DEVICE_VERSION {
            return Err(RepositoryError::Corrupt);
        }
        Ok(disk)
    }
}

impl CertificateTrustStore for LocalCertificateTrustStore {
    fn load(&self, device: DeviceId) -> Result<Option<TrustRecord>, TrustError> {
        let _guard = self.file.lock.lock().map_err(|_| TrustError::Unavailable)?;
        let disk = self.load_disk()?;
        disk.records
            .get(&device.to_string())
            .map(convert_trust)
            .transpose()
    }

    fn save(&self, device: DeviceId, record: &TrustRecord) -> Result<(), TrustError> {
        let _guard = self.file.lock.lock().map_err(|_| TrustError::Unavailable)?;
        let mut disk = self.load_disk()?;
        disk.records.insert(
            device.to_string(),
            DiskTrustRecord {
                host: record.host.as_str().to_owned(),
                certificate_sha256: record.pin.to_hex(),
            },
        );
        self.file.write(&disk).map_err(|_| TrustError::Unavailable)
    }

    fn delete(&self, device: DeviceId) -> Result<(), TrustError> {
        let _guard = self.file.lock.lock().map_err(|_| TrustError::Unavailable)?;
        let mut disk = self.load_disk()?;
        disk.records.remove(&device.to_string());
        self.file.write(&disk).map_err(|_| TrustError::Unavailable)
    }
}

impl LocalCertificateTrustStore {
    fn load_disk(&self) -> Result<DiskTrust, TrustError> {
        let disk: DiskTrust = self
            .file
            .read()
            .map_err(|error| match error.kind() {
                std::io::ErrorKind::InvalidData => TrustError::Corrupt,
                _ => TrustError::Unavailable,
            })?
            .unwrap_or_default();
        if disk.version != TRUST_VERSION {
            return Err(TrustError::Corrupt);
        }
        Ok(disk)
    }
}

fn convert_device(item: DiskDevice) -> Result<SavedDevice, RepositoryError> {
    let wake = item
        .wake
        .and_then(|value| serde_json::from_value::<DiskWake>(value).ok())
        .and_then(|value| {
            let wired = value.wired.map(|mac| mac.parse()).transpose().ok()?;
            let wifi = value.wifi.map(|mac| mac.parse()).transpose().ok()?;
            let active = match value.active.as_deref() {
                None => None,
                Some("wired") => Some(WakeInterface::Wired),
                Some("wifi") => Some(WakeInterface::WiFi),
                _ => return None,
            };
            let wake = WakeConfiguration {
                wired,
                wifi,
                active,
            };
            if wake.active.is_some() && wake.active_mac().is_none() {
                return None;
            }
            Some(wake)
        })
        .unwrap_or_default();
    Ok(SavedDevice {
        id: DeviceId::parse_record_key(&item.id).ok_or(RepositoryError::Corrupt)?,
        label: item.label,
        host: TvHost::parse(&item.host).map_err(|_| RepositoryError::Corrupt)?,
        wake,
    })
}

fn convert_trust(item: &DiskTrustRecord) -> Result<TrustRecord, TrustError> {
    Ok(TrustRecord {
        host: TvHost::parse(&item.host).map_err(|_| TrustError::Corrupt)?,
        pin: CertificatePin::from_hex(&item.certificate_sha256).ok_or(TrustError::Corrupt)?,
    })
}

impl JsonFile {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Mutex::new(()),
        }
    }

    fn read<T: for<'de> Deserialize<'de>>(&self) -> Result<Option<T>, std::io::Error> {
        let file = match File::open(&self.path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        if file.metadata()?.len() > MAX_FILE_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "oversized settings",
            ));
        }
        let mut bytes = Vec::new();
        file.take(MAX_FILE_BYTES + 1).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "oversized settings",
            ));
        }
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid settings"))
    }

    fn write<T: Serialize>(&self, value: &T) -> Result<(), std::io::Error> {
        let parent = self
            .path
            .parent()
            .ok_or_else(|| std::io::Error::other("missing data directory"))?;
        fs::create_dir_all(parent)?;
        fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
        let bytes = serde_json::to_vec(value)
            .map_err(|_| std::io::Error::other("settings encoding failed"))?;
        if bytes.len() as u64 > MAX_FILE_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "oversized settings",
            ));
        }
        let temporary = self
            .path
            .with_extension(format!("tmp-{}", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&temporary)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_and_trust_records_roundtrip_without_a_token_in_preferences() {
        let dir = tempfile::tempdir().unwrap();
        let repository = LocalDeviceRepository::new(dir.path());
        let trust = LocalCertificateTrustStore::new(dir.path());
        let device = SavedDevice {
            id: DeviceId::generate(),
            label: "TV".to_owned(),
            host: TvHost::parse("tv.local").unwrap(),
            wake: WakeConfiguration::default(),
        };
        repository.save(&device).unwrap();
        repository.select(Some(device.id)).unwrap();
        trust
            .save(
                device.id,
                &TrustRecord {
                    host: device.host.clone(),
                    pin: CertificatePin::from_der(b"fake certificate"),
                },
            )
            .unwrap();
        assert_eq!(repository.list().unwrap(), vec![device.clone()]);
        assert_eq!(repository.selected().unwrap(), Some(device.id));
        assert!(trust.load(device.id).unwrap().is_some());
        let preferences = fs::read_to_string(dir.path().join("devices.json")).unwrap();
        assert!(!preferences.contains("token"));
        repository.delete(device.id).unwrap();
        trust.delete(device.id).unwrap();
        assert!(repository.list().unwrap().is_empty());
        assert_eq!(repository.selected().unwrap(), None);
        assert!(trust.load(device.id).unwrap().is_none());
    }

    #[test]
    fn wake_settings_roundtrip_and_legacy_records_remain_selected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("devices.json");
        let id = DeviceId::generate().to_string();
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "selected": id,
                "devices": [{"id": id, "label": "TV", "host": "tv.local"}]
            }))
            .unwrap(),
        )
        .unwrap();
        let repository = LocalDeviceRepository::new(dir.path());
        let mut device = repository.list().unwrap().remove(0);
        assert_eq!(repository.selected().unwrap(), Some(device.id));
        assert_eq!(device.wake, WakeConfiguration::default());
        device.wake = WakeConfiguration {
            wired: Some("02:11:22:33:44:55".parse().unwrap()),
            wifi: None,
            active: Some(WakeInterface::Wired),
        };
        repository.save(&device).unwrap();
        assert_eq!(repository.list().unwrap(), vec![device]);
        let disk: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert_eq!(disk["version"], 2);
    }

    #[test]
    fn malformed_wake_field_does_not_lose_saved_device_or_selection() {
        let dir = tempfile::tempdir().unwrap();
        let id = DeviceId::generate().to_string();
        fs::write(
            dir.path().join("devices.json"),
            serde_json::to_vec(&serde_json::json!({
                "version": 2,
                "selected": id,
                "devices": [{"id": id, "label": "TV", "host": "tv.local",
                    "wake": {"wired": "not-a-mac", "wifi": null, "active": "wired"}}]
            }))
            .unwrap(),
        )
        .unwrap();
        let repository = LocalDeviceRepository::new(dir.path());
        let device = repository.list().unwrap().remove(0);
        assert_eq!(device.wake, WakeConfiguration::default());
        assert_eq!(repository.selected().unwrap(), Some(device.id));
    }

    #[test]
    fn obsolete_verified_actions_are_ignored_and_removed_on_save() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("devices.json");
        let id = DeviceId::generate().to_string();
        let second_id = DeviceId::generate().to_string();
        fs::write(
            &path,
            serde_json::to_vec(&serde_json::json!({
                "version": 1,
                "selected": id,
                "devices": [{
                    "id": id,
                    "label": "TV",
                    "host": "tv.local",
                    "verified_actions": ["Select"]
            }, {
                "id": second_id,
                "label": "Other TV",
                "host": "other.local",
                "verified_actions": ["Enter"]
                }]
            }))
            .unwrap(),
        )
        .unwrap();
        let repository = LocalDeviceRepository::new(dir.path());
        let devices = repository.list().unwrap();
        assert_eq!(devices.len(), 2);
        for device in devices {
            repository.save(&device).unwrap();
        }
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        assert!(saved["devices"]
            .as_array()
            .unwrap()
            .iter()
            .all(|device| device.get("verified_actions").is_none()));
    }
}
