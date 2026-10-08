use crate::domain::{DeviceId, RemoteAction};

use super::device::{DeviceRepository, RepositoryError, SavedDevice};
use super::secret::{PairingToken, SecretError, SecretStore};
use super::target::TvHost;
use super::trust::{CertificatePin, TrustError, TrustRecord, TrustStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupError {
    InvalidLabel,
    Credential(SecretError),
    Trust(TrustError),
    Preferences(RepositoryError),
    PartialCleanup,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconnectError {
    Preferences(RepositoryError),
    Trust(TrustError),
    Credential(SecretError),
    MissingDevice,
    MissingTrust,
    HostChanged,
    PairingRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForgetResult {
    pub credential_removed: bool,
    pub trust_removed: bool,
    pub record_removed: bool,
}

impl ForgetResult {
    pub const fn complete(self) -> bool {
        self.credential_removed && self.trust_removed && self.record_removed
    }
}

pub struct ReconnectMaterial {
    pub device: SavedDevice,
    pub pin: CertificatePin,
    pub token: PairingToken,
}

/// Handles durable setup and recovery. The transport must have verified the
/// same pin and obtained a consent token before `commit_pairing` is called.
pub struct DeviceService<R, S, T> {
    devices: R,
    secrets: S,
    trust: T,
}

impl<R: DeviceRepository, S: SecretStore, T: TrustStore> DeviceService<R, S, T> {
    pub fn new(devices: R, secrets: S, trust: T) -> Self {
        Self {
            devices,
            secrets,
            trust,
        }
    }

    pub fn saved_devices(&self) -> Result<Vec<SavedDevice>, RepositoryError> {
        self.devices.list()
    }

    pub fn selected_device(&self) -> Result<Option<SavedDevice>, RepositoryError> {
        let Some(id) = self.devices.selected()? else {
            return Ok(None);
        };
        self.devices
            .list()?
            .into_iter()
            .find(|device| device.id == id)
            .map(Some)
            .ok_or(RepositoryError::Corrupt)
    }

    pub fn select_saved(&self, id: DeviceId) -> Result<SavedDevice, RepositoryError> {
        let device = self
            .devices
            .list()?
            .into_iter()
            .find(|device| device.id == id)
            .ok_or(RepositoryError::Corrupt)?;
        self.devices.select(Some(id))?;
        Ok(device)
    }

    pub fn commit_pairing(
        &self,
        label: &str,
        host: TvHost,
        pin: CertificatePin,
        token: PairingToken,
    ) -> Result<SavedDevice, SetupError> {
        if label.is_empty()
            || label.len() > 80
            || label.trim() != label
            || label.chars().any(char::is_control)
        {
            return Err(SetupError::InvalidLabel);
        }
        let device = SavedDevice {
            id: DeviceId::generate(),
            label: label.to_owned(),
            host: host.clone(),
            verified_actions: Vec::new(),
        };
        // Keep the prior selected TV until all new records are saved. A failed
        // Keychain write must never create an active paired record.
        self.secrets
            .save(device.id, &token)
            .map_err(SetupError::Credential)?;
        if let Err(error) = self.trust.save(device.id, &TrustRecord { host, pin }) {
            return Err(if self.secrets.delete(device.id).is_ok() {
                SetupError::Trust(error)
            } else {
                SetupError::PartialCleanup
            });
        }
        if let Err(error) = self.devices.save(&device) {
            return Err(if self.cleanup_new(device.id).complete() {
                SetupError::Preferences(error)
            } else {
                SetupError::PartialCleanup
            });
        }
        if let Err(error) = self.devices.select(Some(device.id)) {
            return Err(if self.cleanup_new(device.id).complete() {
                SetupError::Preferences(error)
            } else {
                SetupError::PartialCleanup
            });
        }
        Ok(device)
    }

    pub fn repair(
        &self,
        id: DeviceId,
        label: &str,
        host: TvHost,
        pin: CertificatePin,
        token: PairingToken,
    ) -> Result<SavedDevice, SetupError> {
        if label.is_empty() || label.len() > 80 || label.chars().any(char::is_control) {
            return Err(SetupError::InvalidLabel);
        }
        let mut device = self
            .devices
            .list()
            .map_err(SetupError::Preferences)?
            .into_iter()
            .find(|device| device.id == id)
            .ok_or(SetupError::Preferences(RepositoryError::Corrupt))?;
        let old_host = device.host.clone();
        let old_trust = self.trust.load(id).map_err(SetupError::Trust)?;
        let old_token = self.secrets.load(id).map_err(SetupError::Credential)?;
        let changed_identity = old_host != host
            || old_trust
                .as_ref()
                .is_none_or(|record| record.host != host || record.pin != pin);
        device.label = label.to_owned();
        device.host = host.clone();
        if changed_identity {
            device.verified_actions.clear();
        }

        self.secrets
            .save(id, &token)
            .map_err(SetupError::Credential)?;
        if let Err(error) = self.trust.save(id, &TrustRecord { host, pin }) {
            return Err(
                if self.restore_credentials(id, old_token.as_ref(), old_trust.as_ref()) {
                    SetupError::Trust(error)
                } else {
                    SetupError::PartialCleanup
                },
            );
        }
        if let Err(error) = self.devices.save(&device) {
            return Err(
                if self.restore_credentials(id, old_token.as_ref(), old_trust.as_ref()) {
                    SetupError::Preferences(error)
                } else {
                    SetupError::PartialCleanup
                },
            );
        }
        Ok(device)
    }

    fn restore_credentials(
        &self,
        id: DeviceId,
        token: Option<&PairingToken>,
        trust: Option<&TrustRecord>,
    ) -> bool {
        let token_restored = match token {
            Some(token) => self.secrets.save(id, token).is_ok(),
            None => self.secrets.delete(id).is_ok(),
        };
        let trust_restored = match trust {
            Some(record) => self.trust.save(id, record).is_ok(),
            None => self.trust.delete(id).is_ok(),
        };
        token_restored && trust_restored
    }

    pub fn reconnect_material(&self, id: DeviceId) -> Result<ReconnectMaterial, ReconnectError> {
        let device = self
            .devices
            .list()
            .map_err(ReconnectError::Preferences)?
            .into_iter()
            .find(|device| device.id == id)
            .ok_or(ReconnectError::MissingDevice)?;
        let trust = self
            .trust
            .load(id)
            .map_err(ReconnectError::Trust)?
            .ok_or(ReconnectError::MissingTrust)?;
        if trust.host != device.host {
            return Err(ReconnectError::HostChanged);
        }
        let token = self
            .secrets
            .load(id)
            .map_err(ReconnectError::Credential)?
            .ok_or(ReconnectError::PairingRequired)?;
        Ok(ReconnectMaterial {
            device,
            pin: trust.pin,
            token,
        })
    }

    pub fn save_rotated_token(
        &self,
        id: DeviceId,
        token: &PairingToken,
    ) -> Result<(), SecretError> {
        self.secrets.save(id, token)
    }

    pub fn set_action_verified(
        &self,
        id: DeviceId,
        action: RemoteAction,
        verified: bool,
    ) -> Result<Vec<RemoteAction>, RepositoryError> {
        if action.is_deferred() {
            return Err(RepositoryError::Corrupt);
        }
        let mut device = self
            .devices
            .list()?
            .into_iter()
            .find(|device| device.id == id)
            .ok_or(RepositoryError::Corrupt)?;
        device
            .verified_actions
            .retain(|candidate| *candidate != action);
        if verified {
            device.verified_actions.push(action);
        }
        self.devices.save(&device)?;
        Ok(device.verified_actions)
    }

    pub fn forget(&self, id: DeviceId) -> ForgetResult {
        self.cleanup_new(id)
    }

    fn cleanup_new(&self, id: DeviceId) -> ForgetResult {
        // Attempt every removal even if an earlier store is unavailable.
        let credential_removed = self.secrets.delete(id).is_ok();
        let trust_removed = self.trust.delete(id).is_ok();
        let record_removed = self.devices.delete(id).is_ok();
        ForgetResult {
            credential_removed,
            trust_removed,
            record_removed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    #[derive(Default)]
    struct Memory {
        devices: Vec<SavedDevice>,
        selected: Option<DeviceId>,
        secrets: HashMap<DeviceId, String>,
        trust: HashMap<DeviceId, TrustRecord>,
        fail_secret_save: bool,
        fail_secret_delete: bool,
        fail_trust_save: bool,
        fail_device_save: bool,
        fail_device_select: bool,
        secret_loads: usize,
    }

    #[derive(Clone, Default)]
    struct Fake(Rc<RefCell<Memory>>);

    impl DeviceRepository for Fake {
        fn list(&self) -> Result<Vec<SavedDevice>, RepositoryError> {
            Ok(self.0.borrow().devices.clone())
        }
        fn save(&self, device: &SavedDevice) -> Result<(), RepositoryError> {
            let mut memory = self.0.borrow_mut();
            if memory.fail_device_save {
                return Err(RepositoryError::Unavailable);
            }
            if let Some(existing) = memory.devices.iter_mut().find(|item| item.id == device.id) {
                *existing = device.clone();
            } else {
                memory.devices.push(device.clone());
            }
            Ok(())
        }
        fn delete(&self, id: DeviceId) -> Result<(), RepositoryError> {
            let mut memory = self.0.borrow_mut();
            memory.devices.retain(|device| device.id != id);
            if memory.selected == Some(id) {
                memory.selected = None;
            }
            Ok(())
        }
        fn selected(&self) -> Result<Option<DeviceId>, RepositoryError> {
            Ok(self.0.borrow().selected)
        }
        fn select(&self, id: Option<DeviceId>) -> Result<(), RepositoryError> {
            let mut memory = self.0.borrow_mut();
            if memory.fail_device_select {
                return Err(RepositoryError::Unavailable);
            }
            memory.selected = id;
            Ok(())
        }
    }

    impl SecretStore for Fake {
        fn load(&self, device: DeviceId) -> Result<Option<PairingToken>, SecretError> {
            let mut memory = self.0.borrow_mut();
            memory.secret_loads += 1;
            Ok(memory
                .secrets
                .get(&device)
                .and_then(|value| PairingToken::from_stored(value.clone())))
        }
        fn save(&self, device: DeviceId, token: &PairingToken) -> Result<(), SecretError> {
            let mut memory = self.0.borrow_mut();
            if memory.fail_secret_save {
                return Err(SecretError::Unavailable);
            }
            memory.secrets.insert(device, token.as_str().to_owned());
            Ok(())
        }
        fn delete(&self, device: DeviceId) -> Result<(), SecretError> {
            let mut memory = self.0.borrow_mut();
            if memory.fail_secret_delete {
                return Err(SecretError::Unavailable);
            }
            memory.secrets.remove(&device);
            Ok(())
        }
    }

    impl TrustStore for Fake {
        fn load(&self, device: DeviceId) -> Result<Option<TrustRecord>, TrustError> {
            Ok(self.0.borrow().trust.get(&device).cloned())
        }
        fn save(&self, device: DeviceId, record: &TrustRecord) -> Result<(), TrustError> {
            let mut memory = self.0.borrow_mut();
            if memory.fail_trust_save {
                return Err(TrustError::Unavailable);
            }
            memory.trust.insert(device, record.clone());
            Ok(())
        }
        fn delete(&self, device: DeviceId) -> Result<(), TrustError> {
            self.0.borrow_mut().trust.remove(&device);
            Ok(())
        }
    }

    fn service(fake: &Fake) -> DeviceService<Fake, Fake, Fake> {
        DeviceService::new(fake.clone(), fake.clone(), fake.clone())
    }

    fn commit(service: &DeviceService<Fake, Fake, Fake>) -> Result<SavedDevice, SetupError> {
        service.commit_pairing(
            "TV",
            TvHost::parse("tv.local").unwrap(),
            CertificatePin::from_der(b"certificate"),
            PairingToken::from_stored("token".to_owned()).unwrap(),
        )
    }

    #[test]
    fn keychain_failure_never_saves_or_selects_candidate() {
        let fake = Fake::default();
        fake.0.borrow_mut().fail_secret_save = true;
        assert_eq!(
            commit(&service(&fake)).unwrap_err(),
            SetupError::Credential(SecretError::Unavailable)
        );
        let memory = fake.0.borrow();
        assert!(memory.devices.is_empty());
        assert!(memory.trust.is_empty());
        assert_eq!(memory.selected, None);
    }

    #[test]
    fn trust_failure_rolls_back_keychain_token() {
        let fake = Fake::default();
        fake.0.borrow_mut().fail_trust_save = true;
        assert_eq!(
            commit(&service(&fake)).unwrap_err(),
            SetupError::Trust(TrustError::Unavailable)
        );
        assert!(fake.0.borrow().secrets.is_empty());
    }

    #[test]
    fn failed_device_save_removes_new_credentials_and_preserves_prior_selection() {
        let fake = Fake::default();
        let service = service(&fake);
        let first = commit(&service).unwrap();
        fake.0.borrow_mut().fail_device_save = true;

        assert_eq!(
            commit(&service).unwrap_err(),
            SetupError::Preferences(RepositoryError::Unavailable)
        );
        let memory = fake.0.borrow();
        assert_eq!(memory.selected, Some(first.id));
        assert_eq!(memory.devices.len(), 1);
        assert_eq!(memory.secrets.len(), 1);
        assert_eq!(memory.trust.len(), 1);
    }

    #[test]
    fn failed_selection_removes_candidate_and_keeps_prior_selection() {
        let fake = Fake::default();
        let service = service(&fake);
        let first = commit(&service).unwrap();
        fake.0.borrow_mut().fail_device_select = true;

        assert_eq!(
            commit(&service).unwrap_err(),
            SetupError::Preferences(RepositoryError::Unavailable)
        );
        let memory = fake.0.borrow();
        assert_eq!(memory.selected, Some(first.id));
        assert_eq!(memory.devices.len(), 1);
        assert_eq!(memory.secrets.len(), 1);
        assert_eq!(memory.trust.len(), 1);
    }

    #[test]
    fn changed_host_blocks_token_access_before_reconnect() {
        let fake = Fake::default();
        let service = service(&fake);
        let device = commit(&service).unwrap();
        fake.0.borrow_mut().devices[0].host = TvHost::parse("other.local").unwrap();
        assert!(matches!(
            service.reconnect_material(device.id),
            Err(ReconnectError::HostChanged)
        ));
        assert_eq!(fake.0.borrow().secret_loads, 0);
    }

    #[test]
    fn forget_reports_partial_credential_cleanup_and_still_removes_other_records() {
        let fake = Fake::default();
        let service = service(&fake);
        let device = commit(&service).unwrap();
        fake.0.borrow_mut().fail_secret_delete = true;
        let result = service.forget(device.id);
        assert!(!result.complete());
        assert!(!result.credential_removed);
        assert!(result.trust_removed);
        assert!(result.record_removed);
        let memory = fake.0.borrow();
        assert!(memory.secrets.contains_key(&device.id));
        assert!(memory.trust.is_empty());
        assert!(memory.devices.is_empty());
    }

    #[test]
    fn repair_keeps_record_id_and_resets_verified_keys_when_identity_changes() {
        let fake = Fake::default();
        let service = service(&fake);
        let first = commit(&service).unwrap();
        service
            .set_action_verified(first.id, RemoteAction::Up, true)
            .unwrap();
        let repaired = service
            .repair(
                first.id,
                "Moved TV",
                TvHost::parse("other.local").unwrap(),
                CertificatePin::from_der(b"replacement certificate"),
                PairingToken::from_stored("replacement-token".to_owned()).unwrap(),
            )
            .unwrap();
        assert_eq!(repaired.id, first.id);
        assert!(repaired.verified_actions.is_empty());
        assert_eq!(fake.0.borrow().selected, Some(first.id));
        assert_eq!(
            service.reconnect_material(first.id).unwrap().device.host,
            TvHost::parse("other.local").unwrap()
        );
    }

    #[test]
    fn failed_repair_does_not_overwrite_the_old_token() {
        let fake = Fake::default();
        let service = service(&fake);
        let first = commit(&service).unwrap();
        fake.0.borrow_mut().fail_secret_save = true;
        let result = service.repair(
            first.id,
            "TV",
            first.host.clone(),
            CertificatePin::from_der(b"certificate"),
            PairingToken::from_stored("replacement-token".to_owned()).unwrap(),
        );
        assert_eq!(
            result.unwrap_err(),
            SetupError::Credential(SecretError::Unavailable)
        );
        assert_eq!(
            fake.0.borrow().secrets.get(&first.id).map(String::as_str),
            Some("token")
        );
    }

    #[test]
    fn failed_trust_cleanup_reports_partial_pairing_without_selecting_candidate() {
        let fake = Fake::default();
        fake.0.borrow_mut().fail_trust_save = true;
        fake.0.borrow_mut().fail_secret_delete = true;
        assert_eq!(
            commit(&service(&fake)).unwrap_err(),
            SetupError::PartialCleanup
        );
        let memory = fake.0.borrow();
        assert!(memory.devices.is_empty());
        assert_eq!(memory.selected, None);
        assert_eq!(memory.secrets.len(), 1);
    }

    #[test]
    fn reconnect_requires_trust_and_token_before_returning_material() {
        let fake = Fake::default();
        let service = service(&fake);
        let device = commit(&service).unwrap();
        fake.0.borrow_mut().trust.remove(&device.id);
        assert!(matches!(
            service.reconnect_material(device.id),
            Err(ReconnectError::MissingTrust)
        ));
        assert_eq!(fake.0.borrow().secret_loads, 0);

        fake.0.borrow_mut().trust.insert(
            device.id,
            TrustRecord {
                host: device.host.clone(),
                pin: CertificatePin::from_der(b"certificate"),
            },
        );
        fake.0.borrow_mut().secrets.remove(&device.id);
        assert!(matches!(
            service.reconnect_material(device.id),
            Err(ReconnectError::PairingRequired)
        ));
    }

    #[test]
    fn failed_repair_preferences_write_restores_old_credentials_and_device() {
        let fake = Fake::default();
        let service = service(&fake);
        let device = commit(&service).unwrap();
        fake.0.borrow_mut().fail_device_save = true;
        let result = service.repair(
            device.id,
            "Replacement",
            TvHost::parse("other.local").unwrap(),
            CertificatePin::from_der(b"replacement certificate"),
            PairingToken::from_stored("replacement-token".to_owned()).unwrap(),
        );
        assert_eq!(
            result.unwrap_err(),
            SetupError::Preferences(RepositoryError::Unavailable)
        );
        let memory = fake.0.borrow();
        assert_eq!(memory.devices[0].host, device.host);
        assert_eq!(
            memory.secrets.get(&device.id).map(String::as_str),
            Some("token")
        );
        assert_eq!(memory.trust.get(&device.id).unwrap().host, device.host);
    }

    #[test]
    fn repair_same_identity_preserves_verified_actions() {
        let fake = Fake::default();
        let service = service(&fake);
        let device = commit(&service).unwrap();
        service
            .set_action_verified(device.id, RemoteAction::Up, true)
            .unwrap();
        let repaired = service
            .repair(
                device.id,
                "Renamed TV",
                device.host.clone(),
                CertificatePin::from_der(b"certificate"),
                PairingToken::from_stored("rotated-token".to_owned()).unwrap(),
            )
            .unwrap();
        assert_eq!(repaired.verified_actions, vec![RemoteAction::Up]);
        assert_eq!(repaired.label, "Renamed TV");
        assert_eq!(
            service
                .reconnect_material(device.id)
                .unwrap()
                .token
                .as_str(),
            "rotated-token"
        );
    }
}
