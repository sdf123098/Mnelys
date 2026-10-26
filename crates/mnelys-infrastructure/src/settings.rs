//! The JSON settings store.
//!
//! The shape itself lives in `mnelys_application::settings`; this adapter only owns
//! `settings.json` and the atomic write.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::paths::GamePaths;
use mnelys_application::ports::FileStore;
use mnelys_application::settings::{AppSettings, SettingsStore};
use mnelys_domain::{CoreError, CoreResult};

/// Reads and writes `settings.json`.
pub struct JsonSettingsStore {
    files: Arc<dyn FileStore>,
    paths: GamePaths,
}

impl JsonSettingsStore {
    #[must_use]
    pub fn new(files: Arc<dyn FileStore>, paths: GamePaths) -> Self {
        Self { files, paths }
    }
}

#[async_trait]
impl SettingsStore for JsonSettingsStore {
    /// Loads the settings, falling back to defaults when the file is absent.
    ///
    /// A damaged file is an error rather than silently reset: the user should know
    /// their mirror choice was lost.
    async fn load(&self) -> CoreResult<AppSettings> {
        let path = self.paths.settings_file();
        if !self.files.exists(&path).await {
            return Ok(AppSettings::default());
        }
        let bytes = self.files.read(&path).await?;
        let settings: AppSettings = serde_json::from_slice(&bytes).map_err(|error| {
            CoreError::io(format!("settings file {} is damaged", path.display()))
                .with_details(error.to_string())
        })?;
        settings.check_schema()?;
        Ok(settings)
    }

    async fn save(&self, settings: &AppSettings) -> CoreResult<()> {
        let bytes = serde_json::to_vec_pretty(settings).map_err(|error| {
            CoreError::internal("the settings could not be encoded").with_details(error.to_string())
        })?;
        self.files
            .write_atomic(&self.paths.settings_file(), &bytes)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mnelys_application::mirrors::MirrorPreference;
    use mnelys_application::ports::DirEntry;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryFiles {
        files: Mutex<std::collections::BTreeMap<PathBuf, Vec<u8>>>,
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

        async fn list(&self, _: &Path) -> CoreResult<Vec<DirEntry>> {
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

    #[tokio::test]
    async fn defaults_are_returned_when_nothing_is_stored() {
        let store =
            JsonSettingsStore::new(Arc::new(MemoryFiles::default()), GamePaths::new("/data"));
        let settings = store.load().await.expect("load");
        assert_eq!(settings.mirror, MirrorPreference::Direct);
    }

    #[tokio::test]
    async fn a_saved_mirror_survives_a_reload() {
        let store =
            JsonSettingsStore::new(Arc::new(MemoryFiles::default()), GamePaths::new("/data"));
        let mut settings = store.load().await.expect("load");
        settings.mirror = MirrorPreference::Bmclapi;
        store.save(&settings).await.expect("save");
        assert_eq!(
            store.load().await.expect("load").mirror,
            MirrorPreference::Bmclapi
        );
    }
}
