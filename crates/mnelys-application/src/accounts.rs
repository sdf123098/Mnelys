//! Accounts: Microsoft device-code sign-in, offline identities and Yggdrasil.
//!
//! Secrets never leave this module except as an [`AuthSession`] handed to the launch
//! planner. Passwords are used once and dropped; refresh tokens live in the platform
//! keystore.

use std::sync::Arc;

use mnelys_domain::{
    offline_uuid, validate_offline_username, AccountKind, AccountRecord, AccountSummary,
    AuthSession, CoreError, CoreResult, DeviceCodeChallenge, YggdrasilEndpoint,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::ports::{AccountRepository, Clock, HttpClient, HttpRequest, SecretStore};
use crate::util::join_url;

const DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBL_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const SCOPE: &str = "XboxLive.signin offline_access";
const DEVICE_CODE_LIFETIME_MS: u64 = 15 * 60 * 1000;

/// The Azure application Mnelys signs in with.
///
/// A client id is public by design, but Mnelys does not ship one: an operator must
/// register an app that allows personal Microsoft accounts and public client flows.
#[derive(Debug, Clone, Default)]
pub struct MicrosoftConfig {
    pub client_id: Option<String>,
}

impl MicrosoftConfig {
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            client_id: std::env::var("MNELYS_MS_CLIENT_ID")
                .ok()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredSecret {
    access_token: String,
    #[serde(default)]
    refresh_token: String,
    /// Unix milliseconds; zero means the provider has no known expiry.
    expires_at_ms: u64,
    #[serde(default)]
    xuid: Option<String>,
}

/// Account lifecycle and token custody.
pub struct AccountService {
    repo: Arc<dyn AccountRepository>,
    secrets: Arc<dyn SecretStore>,
    client: Arc<dyn HttpClient>,
    clock: Arc<dyn Clock>,
    microsoft: MicrosoftConfig,
}

impl AccountService {
    #[must_use]
    pub fn new(
        repo: Arc<dyn AccountRepository>,
        secrets: Arc<dyn SecretStore>,
        client: Arc<dyn HttpClient>,
        clock: Arc<dyn Clock>,
        microsoft: MicrosoftConfig,
    ) -> Self {
        Self {
            repo,
            secrets,
            client,
            clock,
            microsoft,
        }
    }

    pub async fn list(&self) -> CoreResult<Vec<AccountSummary>> {
        let selected = self.repo.selected().await?;
        let records = self.repo.list().await?;
        Ok(records
            .iter()
            .map(|record| {
                AccountSummary::from_record(record, selected.as_deref() == Some(record.id.as_str()))
            })
            .collect())
    }

    pub async fn add_offline(&self, username: &str) -> CoreResult<AccountSummary> {
        let username = username.trim();
        validate_offline_username(username)?;
        let uuid = offline_uuid(username);
        if self
            .repo
            .list()
            .await?
            .iter()
            .any(|record| record.kind == AccountKind::Offline && record.uuid == uuid)
        {
            return Err(CoreError::conflict(format!(
                "the offline account `{username}` is already added"
            )));
        }
        let record = AccountRecord::new(
            Uuid::new_v4().to_string(),
            AccountKind::Offline,
            username.to_owned(),
            uuid,
            self.clock.now_ms(),
        );
        self.repo.save(&record).await?;
        self.select_if_empty(&record.id).await?;
        self.summary(&record).await
    }

    pub async fn add_yggdrasil(
        &self,
        auth_url: &str,
        username: &str,
        password: &str,
    ) -> CoreResult<AccountSummary> {
        let endpoint = YggdrasilEndpoint {
            auth_url: auth_url.trim().to_owned(),
        };
        endpoint.validate()?;
        let username = username.trim();
        if username.is_empty() || password.is_empty() {
            return Err(CoreError::invalid_request(
                "a Yggdrasil username and password are required",
            ));
        }
        let client_token = Uuid::new_v4().to_string();
        let url = join_url(&endpoint.auth_url, "authserver/authenticate");
        let body = json!({
            "agent": { "name": "Minecraft", "version": 1 },
            "username": username,
            "password": password,
            "clientToken": client_token,
            "requestUser": false,
        });
        let (status, value) = self.json_outcome(&url, body).await?;
        if !(200..300).contains(&status) {
            return Err(CoreError::unauthorized(
                "the Yggdrasil server rejected the username or password",
            )
            .with_details(error_text(&value)));
        }
        let access_token = required_string(&value, "accessToken")?;
        let selected = value
            .get("selectedProfile")
            .ok_or_else(|| CoreError::unauthorized("the Yggdrasil account has no game profile"))?;
        let uuid = normalize_uuid(&required_string(selected, "id")?);
        let name = required_string(selected, "name")?;

        let record = self
            .upsert(AccountKind::Yggdrasil, &name, &uuid, Some(endpoint.clone()))
            .await?;
        self.store_secret(
            &record.id,
            StoredSecret {
                access_token,
                refresh_token: client_token,
                expires_at_ms: 0,
                xuid: None,
            },
        )?;
        self.repo.select(Some(&record.id)).await?;
        self.summary(&record).await
    }

    pub async fn begin_microsoft(&self) -> CoreResult<DeviceCodeChallenge> {
        let client_id = self.client_id()?;
        let (status, value) = self
            .form_outcome(
                DEVICE_CODE_URL,
                vec![
                    (String::from("client_id"), client_id),
                    (String::from("scope"), String::from(SCOPE)),
                ],
            )
            .await?;
        if !(200..300).contains(&status) {
            return Err(
                CoreError::unauthorized("Microsoft rejected the device-code request")
                    .with_details(error_text(&value)),
            );
        }
        Ok(DeviceCodeChallenge {
            device_code: required_string(&value, "device_code")?,
            user_code: required_string(&value, "user_code")?,
            verification_uri: required_string(&value, "verification_uri")?,
            expires_in: value["expires_in"].as_u64().unwrap_or(900),
            interval: value["interval"].as_u64().unwrap_or(5),
            message: value["message"].as_str().unwrap_or_default().to_owned(),
        })
    }

    /// Polls until the user approves the code, then completes the Xbox chain.
    pub async fn complete_microsoft(
        &self,
        device_code: &str,
        interval_secs: u64,
    ) -> CoreResult<AccountSummary> {
        let client_id = self.client_id()?;
        let mut interval = interval_secs.max(1);
        let deadline = self.clock.now_ms() + DEVICE_CODE_LIFETIME_MS;
        loop {
            if self.clock.now_ms() > deadline {
                return Err(CoreError::unauthorized(
                    "the Microsoft device code expired; start sign-in again",
                ));
            }
            let (status, value) = self
                .form_outcome(
                    TOKEN_URL,
                    vec![
                        (
                            String::from("grant_type"),
                            String::from("urn:ietf:params:oauth:grant-type:device_code"),
                        ),
                        (String::from("client_id"), client_id.clone()),
                        (String::from("device_code"), device_code.to_owned()),
                    ],
                )
                .await?;
            if (200..300).contains(&status) {
                let access_token = required_string(&value, "access_token")?;
                let refresh_token = required_string(&value, "refresh_token")?;
                let expires_in = value["expires_in"].as_u64().unwrap_or(3600);
                return self
                    .finish_microsoft(&access_token, &refresh_token, expires_in)
                    .await;
            }
            match value["error"].as_str() {
                Some("authorization_pending") => {}
                Some("slow_down") => interval += 5,
                Some("expired_token") => {
                    return Err(CoreError::unauthorized(
                        "the Microsoft device code expired; start sign-in again",
                    ))
                }
                Some("authorization_declined") | Some("access_denied") => {
                    return Err(CoreError::unauthorized("Microsoft sign-in was declined"))
                }
                Some(other) => {
                    return Err(CoreError::unauthorized(format!(
                        "Microsoft sign-in failed: {other}"
                    ))
                    .with_details(error_text(&value)))
                }
                None => {
                    return Err(CoreError::network(
                        "Microsoft returned an unexpected token response",
                    )
                    .with_details(error_text(&value)))
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    }

    pub async fn select(&self, id: &str) -> CoreResult<AccountSummary> {
        let record = self.record(id).await?;
        self.repo.select(Some(&record.id)).await?;
        self.summary(&record).await
    }

    pub async fn remove(&self, id: &str) -> CoreResult<()> {
        let record = self.record(id).await?;
        let _ = self.secrets.delete(&secret_key(&record.id));
        self.repo.delete(&record.id).await?;
        if self.repo.selected().await?.as_deref() == Some(record.id.as_str()) {
            self.repo.select(None).await?;
        }
        Ok(())
    }

    /// Forces a token refresh, so the accounts page can repair an expired session.
    pub async fn refresh(&self, id: &str) -> CoreResult<AccountSummary> {
        let record = self.record(id).await?;
        match record.kind {
            AccountKind::Offline => self.summary(&record).await,
            AccountKind::Microsoft => {
                let (record, _) = self.refresh_microsoft(&record).await?;
                self.summary(&record).await
            }
            AccountKind::Yggdrasil => {
                let record = self.refresh_yggdrasil(&record).await?;
                self.summary(&record).await
            }
        }
    }

    /// The identity to launch with. `None` means the selected account.
    pub async fn session(&self, account_id: Option<&str>) -> CoreResult<AuthSession> {
        let record = match account_id {
            Some(id) => self.record(id).await?,
            None => match self.repo.selected().await? {
                Some(id) => self.record(&id).await?,
                None => {
                    return Err(CoreError::unavailable(
                        "no account is selected; add one before launching",
                    ))
                }
            },
        };
        let (mut record, session) = match record.kind {
            AccountKind::Offline => {
                let session = AuthSession {
                    account_id: record.id.clone(),
                    username: record.username.clone(),
                    uuid: record.uuid.clone(),
                    access_token: String::from("0"),
                    user_type: String::from("legacy"),
                    xuid: None,
                };
                (record, session)
            }
            AccountKind::Microsoft => {
                let (record, secret) = self.ensure_microsoft(&record).await?;
                let session = AuthSession {
                    account_id: record.id.clone(),
                    username: record.username.clone(),
                    uuid: record.uuid.clone(),
                    access_token: secret.access_token,
                    user_type: String::from("msa"),
                    xuid: secret.xuid,
                };
                (record, session)
            }
            AccountKind::Yggdrasil => {
                let secret = self
                    .load_secret(&record.id)?
                    .ok_or_else(|| CoreError::unauthorized("sign in to this account again"))?;
                let session = AuthSession {
                    account_id: record.id.clone(),
                    username: record.username.clone(),
                    uuid: record.uuid.clone(),
                    access_token: secret.access_token,
                    user_type: String::from("mojang"),
                    xuid: None,
                };
                (record, session)
            }
        };
        record.last_used_ms = Some(self.clock.now_ms());
        self.repo.save(&record).await?;
        Ok(session)
    }

    async fn finish_microsoft(
        &self,
        access_token: &str,
        refresh_token: &str,
        expires_in: u64,
    ) -> CoreResult<AccountSummary> {
        let (mc_token, xuid, uuid, name) = self.xbox_chain(access_token).await?;
        let record = self
            .upsert(AccountKind::Microsoft, &name, &uuid, None)
            .await?;
        self.store_secret(
            &record.id,
            StoredSecret {
                access_token: mc_token,
                refresh_token: refresh_token.to_owned(),
                expires_at_ms: self.clock.now_ms() + expires_in.saturating_mul(1000),
                xuid: Some(xuid),
            },
        )?;
        self.repo.select(Some(&record.id)).await?;
        self.summary(&record).await
    }

    async fn ensure_microsoft(
        &self,
        record: &AccountRecord,
    ) -> CoreResult<(AccountRecord, StoredSecret)> {
        let secret = self
            .load_secret(&record.id)?
            .ok_or_else(|| CoreError::unauthorized("sign in to this account again"))?;
        if secret.expires_at_ms > self.clock.now_ms() + 60_000 {
            return Ok((record.clone(), secret));
        }
        self.refresh_microsoft(record).await
    }

    async fn refresh_microsoft(
        &self,
        record: &AccountRecord,
    ) -> CoreResult<(AccountRecord, StoredSecret)> {
        let client_id = self.client_id()?;
        let stale = self
            .load_secret(&record.id)?
            .ok_or_else(|| CoreError::unauthorized("sign in to this account again"))?;
        if stale.refresh_token.is_empty() {
            return Err(CoreError::unauthorized(
                "the Microsoft session has no refresh token; sign in again",
            ));
        }
        let (status, value) = self
            .form_outcome(
                TOKEN_URL,
                vec![
                    (String::from("grant_type"), String::from("refresh_token")),
                    (String::from("client_id"), client_id),
                    (String::from("refresh_token"), stale.refresh_token.clone()),
                    (String::from("scope"), String::from(SCOPE)),
                ],
            )
            .await?;
        if !(200..300).contains(&status) {
            return Err(
                CoreError::unauthorized("the Microsoft session expired; sign in again")
                    .with_details(error_text(&value)),
            );
        }
        let access_token = required_string(&value, "access_token")?;
        let refresh_token = value["refresh_token"]
            .as_str()
            .unwrap_or(&stale.refresh_token)
            .to_owned();
        let expires_in = value["expires_in"].as_u64().unwrap_or(3600);
        let (mc_token, xuid, uuid, name) = self.xbox_chain(&access_token).await?;

        let mut updated = record.clone();
        updated.username = name;
        updated.uuid = uuid;
        self.repo.save(&updated).await?;
        let secret = StoredSecret {
            access_token: mc_token,
            refresh_token,
            expires_at_ms: self.clock.now_ms() + expires_in.saturating_mul(1000),
            xuid: Some(xuid),
        };
        self.store_secret(&updated.id, secret.clone())?;
        Ok((updated, secret))
    }

    async fn refresh_yggdrasil(&self, record: &AccountRecord) -> CoreResult<AccountRecord> {
        let endpoint = record
            .yggdrasil
            .clone()
            .ok_or_else(|| CoreError::unauthorized("this account has no Yggdrasil endpoint"))?;
        let secret = self
            .load_secret(&record.id)?
            .ok_or_else(|| CoreError::unauthorized("sign in to this account again"))?;
        let url = join_url(&endpoint.auth_url, "authserver/refresh");
        let (status, value) = self
            .json_outcome(
                &url,
                json!({
                    "accessToken": secret.access_token,
                    "clientToken": secret.refresh_token,
                    "requestUser": false,
                }),
            )
            .await?;
        if !(200..300).contains(&status) {
            return Err(
                CoreError::unauthorized("the Yggdrasil session expired; sign in again")
                    .with_details(error_text(&value)),
            );
        }
        let access_token = required_string(&value, "accessToken")?;
        let updated = StoredSecret {
            access_token,
            refresh_token: value["clientToken"]
                .as_str()
                .unwrap_or(&secret.refresh_token)
                .to_owned(),
            expires_at_ms: 0,
            xuid: None,
        };
        self.store_secret(&record.id, updated)?;
        Ok(record.clone())
    }

    /// Runs Xbox Live, XSTS and Minecraft services, returning the game token and profile.
    async fn xbox_chain(
        &self,
        microsoft_token: &str,
    ) -> CoreResult<(String, String, String, String)> {
        let (status, xbl) = self
            .json_outcome(
                XBL_URL,
                json!({
                    "Properties": {
                        "AuthMethod": "RPS",
                        "SiteName": "user.auth.xboxlive.com",
                        "RpsTicket": format!("d={microsoft_token}")
                    },
                    "RelyingParty": "http://auth.xboxlive.com",
                    "TokenType": "JWT"
                }),
            )
            .await?;
        if !(200..300).contains(&status) {
            return Err(
                CoreError::unauthorized("Xbox Live rejected the Microsoft token")
                    .with_details(error_text(&xbl)),
            );
        }
        let xbl_token = required_string(&xbl, "Token")?;

        let (status, xsts) = self
            .json_outcome(
                XSTS_URL,
                json!({
                    "Properties": {
                        "SandboxId": "RETAIL",
                        "UserTokens": [xbl_token]
                    },
                    "RelyingParty": "rp://api.minecraftservices.com/",
                    "TokenType": "JWT"
                }),
            )
            .await?;
        if !(200..300).contains(&status) {
            return Err(xsts_error(&xsts));
        }
        let xsts_token = required_string(&xsts, "Token")?;
        let xuid = xsts
            .pointer("/DisplayClaims/xui/0/uhs")
            .and_then(Value::as_str)
            .ok_or_else(|| CoreError::unauthorized("Xbox Live did not return a user hash"))?
            .to_owned();

        let (status, login) = self
            .json_outcome(
                MC_LOGIN_URL,
                json!({ "identityToken": format!("XBL3.0 x={xuid};{xsts_token}") }),
            )
            .await?;
        if !(200..300).contains(&status) {
            return Err(
                CoreError::unauthorized("Minecraft services rejected the Xbox token")
                    .with_details(error_text(&login)),
            );
        }
        let access_token = required_string(&login, "access_token")?;

        let response = self
            .client
            .execute(
                HttpRequest::get(MC_PROFILE_URL)
                    .with_header("Authorization", format!("Bearer {access_token}")),
            )
            .await?;
        let profile: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
        if response.status == 404 {
            return Err(CoreError::unauthorized(
                "this Microsoft account does not own Minecraft: Java Edition",
            ));
        }
        if !(200..300).contains(&response.status) {
            return Err(
                CoreError::unauthorized("Minecraft services did not return a profile")
                    .with_details(error_text(&profile)),
            );
        }
        let uuid = normalize_uuid(&required_string(&profile, "id")?);
        let name = required_string(&profile, "name")?;
        Ok((access_token, xuid, uuid, name))
    }

    async fn upsert(
        &self,
        kind: AccountKind,
        name: &str,
        uuid: &str,
        yggdrasil: Option<YggdrasilEndpoint>,
    ) -> CoreResult<AccountRecord> {
        let existing = self.repo.list().await?.into_iter().find(|record| {
            record.kind == kind
                && record.uuid == uuid
                && record.yggdrasil.as_ref().map(|endpoint| &endpoint.auth_url)
                    == yggdrasil.as_ref().map(|endpoint| &endpoint.auth_url)
        });
        let mut record = existing.unwrap_or_else(|| {
            AccountRecord::new(
                Uuid::new_v4().to_string(),
                kind,
                name.to_owned(),
                uuid.to_owned(),
                self.clock.now_ms(),
            )
        });
        record.username = name.to_owned();
        record.uuid = uuid.to_owned();
        if yggdrasil.is_some() {
            record.yggdrasil = yggdrasil;
        }
        self.repo.save(&record).await?;
        Ok(record)
    }

    async fn select_if_empty(&self, id: &str) -> CoreResult<()> {
        if self.repo.selected().await?.is_none() {
            self.repo.select(Some(id)).await?;
        }
        Ok(())
    }

    async fn record(&self, id: &str) -> CoreResult<AccountRecord> {
        self.repo
            .get(id)
            .await?
            .ok_or_else(|| CoreError::unavailable(format!("account `{id}` does not exist")))
    }

    async fn summary(&self, record: &AccountRecord) -> CoreResult<AccountSummary> {
        let selected = self.repo.selected().await?;
        Ok(AccountSummary::from_record(
            record,
            selected.as_deref() == Some(record.id.as_str()),
        ))
    }

    fn load_secret(&self, id: &str) -> CoreResult<Option<StoredSecret>> {
        self.secrets
            .get(&secret_key(id))?
            .map(|value| {
                serde_json::from_str(&value).map_err(|error| {
                    CoreError::internal("the stored account secret is unreadable")
                        .with_details(error.to_string())
                })
            })
            .transpose()
    }

    fn store_secret(&self, id: &str, secret: StoredSecret) -> CoreResult<()> {
        let encoded = serde_json::to_string(&secret).map_err(|error| {
            CoreError::internal("could not encode the account secret")
                .with_details(error.to_string())
        })?;
        self.secrets.put(&secret_key(id), &encoded)
    }

    fn client_id(&self) -> CoreResult<String> {
        self.microsoft.client_id.clone().ok_or_else(|| {
            CoreError::unavailable(
                "Microsoft sign-in is not configured; set MNELYS_MS_CLIENT_ID to an Azure client id",
            )
        })
    }

    async fn form_outcome(
        &self,
        url: &str,
        form: Vec<(String, String)>,
    ) -> CoreResult<(u16, Value)> {
        let response = self
            .client
            .execute(HttpRequest::post_form(url, form))
            .await?;
        let value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
        Ok((response.status, value))
    }

    async fn json_outcome(&self, url: &str, body: Value) -> CoreResult<(u16, Value)> {
        let response = self
            .client
            .execute(HttpRequest::post_json(url, body))
            .await?;
        let value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
        Ok((response.status, value))
    }
}

fn secret_key(account_id: &str) -> String {
    format!("mnelys.account.{account_id}")
}

fn required_string(value: &Value, key: &str) -> CoreResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| {
            CoreError::unauthorized(format!("the authentication response has no `{key}`"))
        })
}

fn error_text(value: &Value) -> String {
    let description = value
        .get("error_description")
        .or_else(|| value.get("errorMessage"))
        .or_else(|| value.get("error"))
        .and_then(Value::as_str)
        .unwrap_or("no details");
    description.chars().take(300).collect()
}

fn xsts_error(value: &Value) -> CoreError {
    let code = value.get("XErr").and_then(Value::as_u64).unwrap_or(0);
    let message = match code {
        2148916233 => "this Microsoft account has no Xbox profile; sign in to xbox.com once",
        2148916235 => "Xbox Live is not available in this account's region",
        2148916238 => "this is a child account and must be added to a Microsoft family",
        _ => "Xbox Live refused the sign-in",
    };
    CoreError::unauthorized(message).with_details(error_text(value))
}

/// Normalizes a Minecraft profile id to the dashed lowercase form.
#[must_use]
pub fn normalize_uuid(value: &str) -> String {
    let hex: String = value
        .chars()
        .filter(char::is_ascii_hexdigit)
        .map(|character| character.to_ascii_lowercase())
        .collect();
    if hex.len() == 32 {
        format!(
            "{}-{}-{}-{}-{}",
            &hex[0..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..32]
        )
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uuids_are_normalized_to_dashes() {
        assert_eq!(
            normalize_uuid("B50AD385829D3141A2167E7D7539BA7F"),
            "b50ad385-829d-3141-a216-7e7d7539ba7f"
        );
        assert_eq!(normalize_uuid("not-a-uuid"), "not-a-uuid");
    }

    #[test]
    fn xsts_errors_are_explained() {
        let error = xsts_error(&json!({ "XErr": 2148916233u64 }));
        assert!(error.message.contains("Xbox profile"));
    }
}
