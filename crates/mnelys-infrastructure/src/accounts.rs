//! The account index stored in `accounts.json`.
//!
//! Only public identity data lives here. Tokens are kept in the platform keystore and
//! keyed by the account id, so losing this file never exposes a credential.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::paths::GamePaths;
use mnelys_application::ports::{AccountRepository, FileStore};
use mnelys_domain::{AccountRecord, CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountsDocument {
    schema_version: u32,
    #[serde(default)]
    selected_id: Option<String>,
    #[serde(default)]
    accounts: Vec<AccountRecord>,
}

impl Default for AccountsDocument {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            selected_id: None,
            accounts: Vec::new(),
        }
    }
}

/// A [`AccountRepository`] over one JSON document.
pub struct JsonAccountRepository {
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    lock: Mutex<()>,
}

impl JsonAccountRepository {
    #[must_use]
    pub fn new(files: Arc<dyn FileStore>, paths: GamePaths) -> Self {
        Self {
            files,
            paths,
            lock: Mutex::new(()),
        }
    }

    async fn load(&self) -> CoreResult<AccountsDocument> {
        let path = self.paths.accounts_file();
        if !self.files.exists(&path).await {
            return Ok(AccountsDocument::default());
        }
        let bytes = self.files.read(&path).await?;
        let document: AccountsDocument = serde_json::from_slice(&bytes).map_err(|error| {
            CoreError::io(format!("account index {} is damaged", path.display()))
                .with_details(error.to_string())
        })?;
        if document.schema_version > SCHEMA_VERSION {
            return Err(CoreError::internal(format!(
                "the account index uses schema {} but this build understands {}",
                document.schema_version, SCHEMA_VERSION
            )));
        }
        Ok(document)
    }

    async fn store(&self, document: &AccountsDocument) -> CoreResult<()> {
        let bytes = serde_json::to_vec_pretty(document).map_err(|error| {
            CoreError::internal("the account index could not be encoded")
                .with_details(error.to_string())
        })?;
        self.files
            .write_atomic(&self.paths.accounts_file(), &bytes)
            .await
    }
}

#[async_trait]
impl AccountRepository for JsonAccountRepository {
    async fn list(&self) -> CoreResult<Vec<AccountRecord>> {
        Ok(self.load().await?.accounts)
    }

    async fn get(&self, id: &str) -> CoreResult<Option<AccountRecord>> {
        Ok(self
            .load()
            .await?
            .accounts
            .into_iter()
            .find(|record| record.id == id))
    }

    async fn save(&self, record: &AccountRecord) -> CoreResult<()> {
        let _guard = self.lock.lock().await;
        let mut document = self.load().await?;
        match document
            .accounts
            .iter_mut()
            .find(|existing| existing.id == record.id)
        {
            Some(existing) => *existing = record.clone(),
            None => document.accounts.push(record.clone()),
        }
        self.store(&document).await
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        let _guard = self.lock.lock().await;
        let mut document = self.load().await?;
        document.accounts.retain(|record| record.id != id);
        if document.selected_id.as_deref() == Some(id) {
            document.selected_id = document.accounts.first().map(|record| record.id.clone());
        }
        self.store(&document).await
    }

    async fn selected(&self) -> CoreResult<Option<String>> {
        let document = self.load().await?;
        Ok(document
            .selected_id
            .filter(|id| document.accounts.iter().any(|record| &record.id == id)))
    }

    async fn select(&self, id: Option<&str>) -> CoreResult<()> {
        let _guard = self.lock.lock().await;
        let mut document = self.load().await?;
        if let Some(id) = id {
            if !document.accounts.iter().any(|record| record.id == id) {
                return Err(CoreError::unavailable(format!(
                    "account `{id}` does not exist"
                )));
            }
            document.selected_id = Some(id.to_owned());
        } else {
            document.selected_id = None;
        }
        self.store(&document).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mnelys_domain::{AccountKind, AccountRecord};
    use std::path::{Path, PathBuf};
    use std::sync::Mutex as StdMutex;

    #[derive(Default)]
    struct MemoryFiles {
        files: StdMutex<std::collections::BTreeMap<PathBuf, Vec<u8>>>,
    }

    #[async_trait]
    impl FileStore for MemoryFiles {
        async fn read(&self, path: &Path) -> CoreResult<Vec<u8>> {
            self.files
                .lock()
                .expect("lock")
                .get(path)
                .cloned()
                .ok_or_else(|| CoreError::io("missing"))
        }

        async fn write_atomic(&self, path: &Path, data: &[u8]) -> CoreResult<()> {
            self.files
                .lock()
                .expect("lock")
                .insert(path.to_path_buf(), data.to_vec());
            Ok(())
        }

        async fn exists(&self, path: &Path) -> bool {
            self.files.lock().expect("lock").contains_key(path)
        }

        async fn remove(&self, path: &Path) -> CoreResult<()> {
            self.files.lock().expect("lock").remove(path);
            Ok(())
        }

        async fn remove_dir_all(&self, _: &Path) -> CoreResult<()> {
            Ok(())
        }

        async fn create_dir_all(&self, _: &Path) -> CoreResult<()> {
            Ok(())
        }

        async fn list(&self, _: &Path) -> CoreResult<Vec<mnelys_application::ports::DirEntry>> {
            Ok(Vec::new())
        }

        async fn len(&self, _: &Path) -> Option<u64> {
            None
        }

        async fn copy(&self, _: &Path, _: &Path) -> CoreResult<()> {
            Ok(())
        }

        async fn extract_zip(&self, _: &Path, _: &Path, _: &[String]) -> CoreResult<usize> {
            Ok(0)
        }
    }

    fn record(id: &str) -> AccountRecord {
        AccountRecord::new(
            id.to_owned(),
            AccountKind::Offline,
            String::from("Steve"),
            String::from("00000000-0000-3000-8000-000000000000"),
            1,
        )
    }

    #[tokio::test]
    async fn accounts_are_saved_and_selected() {
        let repo =
            JsonAccountRepository::new(Arc::new(MemoryFiles::default()), GamePaths::new("/data"));
        repo.save(&record("a")).await.expect("save");
        repo.save(&record("b")).await.expect("save");
        repo.select(Some("b")).await.expect("select");
        assert_eq!(
            repo.selected().await.expect("selected"),
            Some(String::from("b"))
        );

        repo.delete("b").await.expect("delete");
        assert_eq!(
            repo.selected().await.expect("selected"),
            Some(String::from("a"))
        );
        assert_eq!(repo.list().await.expect("list").len(), 1);
    }

    #[tokio::test]
    async fn selecting_an_unknown_account_fails() {
        let repo =
            JsonAccountRepository::new(Arc::new(MemoryFiles::default()), GamePaths::new("/data"));
        assert!(repo.select(Some("missing")).await.is_err());
    }
}
