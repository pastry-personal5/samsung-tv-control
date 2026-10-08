use security_framework::passwords::{
    delete_generic_password, get_generic_password, set_generic_password,
};

use crate::application::credential_store::{CredentialStore, PairingToken, SecretError};
use crate::domain::DeviceId;

const SERVICE: &str = "dev.samsungtvremote.local.pairing";
const ITEM_NOT_FOUND: i32 = -25300;

#[derive(Debug, Default)]
pub struct KeychainCredentialStore;

impl CredentialStore for KeychainCredentialStore {
    fn load(&self, device: DeviceId) -> Result<Option<PairingToken>, SecretError> {
        match get_generic_password(SERVICE, &device.to_string()) {
            Ok(bytes) => {
                let value = String::from_utf8(bytes).map_err(|_| SecretError::InvalidToken)?;
                PairingToken::from_stored(value)
                    .map(Some)
                    .ok_or(SecretError::InvalidToken)
            }
            Err(error) if error.code() == ITEM_NOT_FOUND => Ok(None),
            Err(_) => Err(SecretError::Unavailable),
        }
    }

    fn save(&self, device: DeviceId, token: &PairingToken) -> Result<(), SecretError> {
        set_generic_password(SERVICE, &device.to_string(), token.as_str().as_bytes())
            .map_err(|_| SecretError::Unavailable)
    }

    fn delete(&self, device: DeviceId) -> Result<(), SecretError> {
        match delete_generic_password(SERVICE, &device.to_string()) {
            Ok(()) => Ok(()),
            Err(error) if error.code() == ITEM_NOT_FOUND => Ok(()),
            Err(_) => Err(SecretError::Unavailable),
        }
    }
}
