//! Platform keystore adapter.
//!
//! Credentials never touch `accounts.json`. On Windows this is the credential manager,
//! on macOS the Keychain and on Linux the Secret Service. A machine without a usable
//! keystore reports an error instead of silently falling back to plaintext storage.

use keyring::Entry;
use mnelys_application::ports::SecretStore;
use mnelys_domain::{CoreError, CoreResult};

/// The service name entries are filed under.
pub const SERVICE: &str = "dev.mnelys.account";

/// A [`SecretStore`] over the operating system keystore.
#[derive(Debug, Clone)]
pub struct KeyringSecretStore {
    service: String,
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyringSecretStore {
    #[must_use]
    pub fn new() -> Self {
        Self {
            service: SERVICE.to_owned(),
        }
    }

    #[must_use]
    pub fn with_service(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry(&self, key: &str) -> CoreResult<Entry> {
        if key.is_empty() || key.len() > 200 {
            return Err(CoreError::invalid_request(
                "a secret key must be between 1 and 200 characters",
            ));
        }
        Entry::new(&self.service, key).map_err(|error| {
            CoreError::unavailable("the platform keystore is not available")
                .with_details(error.to_string())
        })
    }
}

impl SecretStore for KeyringSecretStore {
    fn put(&self, key: &str, secret: &str) -> CoreResult<()> {
        self.entry(key)?.set_password(secret).map_err(|error| {
            CoreError::io("the credential could not be stored").with_details(error.to_string())
        })
    }

    fn get(&self, key: &str) -> CoreResult<Option<String>> {
        match self.entry(key)?.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => {
                Err(CoreError::io("the credential could not be read")
                    .with_details(error.to_string()))
            }
        }
    }

    fn delete(&self, key: &str) -> CoreResult<()> {
        match self.entry(key)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(CoreError::io("the credential could not be removed")
                .with_details(error.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_key_is_rejected_before_touching_the_keystore() {
        let store = KeyringSecretStore::new();
        let error = store.get("").expect_err("empty key");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::InvalidRequest);
    }
}
