//! Accounts: the identities the launcher can start a game with.
//!
//! Records here hold no secrets. Access and refresh tokens live in the platform
//! keystore, keyed by [`AccountRecord::id`]; an offline account has no secret at all.

use serde::{Deserialize, Serialize};

use crate::CoreError;

/// Where an account's identity comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountKind {
    Microsoft,
    Offline,
    Yggdrasil,
}

impl AccountKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Microsoft => "microsoft",
            Self::Offline => "offline",
            Self::Yggdrasil => "yggdrasil",
        }
    }

    pub fn parse(value: &str) -> Result<Self, CoreError> {
        match value {
            "microsoft" => Ok(Self::Microsoft),
            "offline" => Ok(Self::Offline),
            "yggdrasil" => Ok(Self::Yggdrasil),
            other => Err(CoreError::invalid_request(format!(
                "`{other}` is not a supported account kind"
            ))),
        }
    }

    /// Whether the account authenticates against a remote service.
    #[must_use]
    pub const fn is_online(self) -> bool {
        matches!(self, Self::Microsoft | Self::Yggdrasil)
    }
}

/// A third-party Yggdrasil authentication endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YggdrasilEndpoint {
    pub auth_url: String,
}

impl YggdrasilEndpoint {
    /// Accepts `https`, plus `http` on loopback for local test servers.
    pub fn validate(&self) -> Result<(), CoreError> {
        let url = self.auth_url.trim();
        let https = url.starts_with("https://");
        let loopback = url.starts_with("http://127.0.0.1")
            || url.starts_with("http://localhost")
            || url.starts_with("http://[::1]");
        if https || loopback {
            if url.len() > 256 || url.chars().any(char::is_whitespace) {
                return Err(CoreError::invalid_request(
                    "the Yggdrasil auth URL is not a clean URL",
                ));
            }
            Ok(())
        } else {
            Err(CoreError::invalid_request(
                "the Yggdrasil auth URL must use https",
            ))
        }
    }
}

/// The persisted account. Written as JSON next to the account list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRecord {
    pub schema_version: u32,
    /// Stable local identifier; tokens are stored under this key.
    pub id: String,
    pub kind: AccountKind,
    /// The name the game receives. Not necessarily unique.
    pub username: String,
    /// The game profile UUID, lowercase dashed form.
    pub uuid: String,
    #[serde(default)]
    pub yggdrasil: Option<YggdrasilEndpoint>,
    pub added_at_ms: u64,
    #[serde(default)]
    pub last_used_ms: Option<u64>,
}

impl AccountRecord {
    pub const SCHEMA_VERSION: u32 = 1;

    #[must_use]
    pub fn new(
        id: String,
        kind: AccountKind,
        username: String,
        uuid: String,
        added_at_ms: u64,
    ) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            id,
            kind,
            username,
            uuid,
            yggdrasil: None,
            added_at_ms,
            last_used_ms: None,
        }
    }

    pub fn check_schema(&self) -> Result<(), CoreError> {
        if self.schema_version > Self::SCHEMA_VERSION {
            return Err(CoreError::internal(format!(
                "account `{}` uses schema {} but this build understands {}",
                self.id,
                self.schema_version,
                Self::SCHEMA_VERSION
            )));
        }
        Ok(())
    }
}

/// What the accounts page displays. Never includes a token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountSummary {
    pub id: String,
    pub kind: AccountKind,
    pub name: String,
    pub uuid: String,
    pub selected: bool,
    pub added_at_ms: u64,
    #[serde(default)]
    pub last_used_ms: Option<u64>,
}

impl AccountSummary {
    #[must_use]
    pub fn from_record(record: &AccountRecord, selected: bool) -> Self {
        Self {
            id: record.id.clone(),
            kind: record.kind,
            name: record.username.clone(),
            uuid: record.uuid.clone(),
            selected,
            added_at_ms: record.added_at_ms,
            last_used_ms: record.last_used_ms,
        }
    }
}

/// A live, in-memory identity used to build a launch command line.
///
/// Deliberately not `Serialize`: no code path should ever hand the token to the
/// frontend or write it into a log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthSession {
    pub account_id: String,
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    /// `msa`, `legacy` or `mojang`; the game passes this through to session servers.
    pub user_type: String,
    pub xuid: Option<String>,
}

/// The device-code challenge a user approves in a browser.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceCodeChallenge {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_https_yggdrasil_endpoint_is_accepted() {
        let endpoint = YggdrasilEndpoint {
            auth_url: String::from("https://littleskin.cn/api/yggdrasil"),
        };
        assert!(endpoint.validate().is_ok());
    }

    #[test]
    fn plain_http_is_only_allowed_on_loopback() {
        let loopback = YggdrasilEndpoint {
            auth_url: String::from("http://127.0.0.1:8080/api"),
        };
        assert!(loopback.validate().is_ok());
        let remote = YggdrasilEndpoint {
            auth_url: String::from("http://example.com/api"),
        };
        assert!(remote.validate().is_err());
    }

    #[test]
    fn only_remote_kinds_are_online() {
        assert!(AccountKind::Microsoft.is_online());
        assert!(AccountKind::Yggdrasil.is_online());
        assert!(!AccountKind::Offline.is_online());
    }

    #[test]
    fn a_summary_never_contains_secrets() {
        let record = AccountRecord::new(
            String::from("local-1"),
            AccountKind::Microsoft,
            String::from("Steve"),
            String::from("00000000-0000-0000-0000-000000000000"),
            1,
        );
        let json = serde_json::to_value(AccountSummary::from_record(&record, true)).expect("json");
        assert!(json.get("accessToken").is_none());
        assert_eq!(json["selected"], true);
    }
}
