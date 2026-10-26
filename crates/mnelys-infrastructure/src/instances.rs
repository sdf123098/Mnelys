//! Instance records stored as one JSON file per instance.
//!
//! The file lives inside the instance directory, so copying an instance directory
//! carries its own record. The repository only manages records; deleting the game
//! files stays the instance service's job.

use std::sync::Arc;

use async_trait::async_trait;
use mnelys_application::paths::GamePaths;
use mnelys_application::ports::{FileStore, InstanceRepository};
use mnelys_domain::{CoreError, CoreResult, InstanceRecord};

/// A [`InstanceRepository`] over `<root>/instances/<id>/instance.json`.
pub struct JsonInstanceRepository {
    files: Arc<dyn FileStore>,
    paths: GamePaths,
}

impl JsonInstanceRepository {
    #[must_use]
    pub fn new(files: Arc<dyn FileStore>, paths: GamePaths) -> Self {
        Self { files, paths }
    }
}

fn validate_id(id: &str) -> CoreResult<()> {
    let safe = !id.is_empty()
        && id.len() <= 48
        && id.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        });
    if safe {
        Ok(())
    } else {
        Err(CoreError::invalid_request(format!(
            "`{id}` is not a valid instance id"
        )))
    }
}

#[async_trait]
impl InstanceRepository for JsonInstanceRepository {
    async fn list(&self) -> CoreResult<Vec<InstanceRecord>> {
        let directory = self.paths.instances_dir();
        let mut records = Vec::new();
        for entry in self.files.list(&directory).await? {
            if !entry.is_dir {
                continue;
            }
            let id = entry.name;
            if validate_id(&id).is_err() {
                continue;
            }
            let path = self.paths.instance_record(&id);
            if !self.files.exists(&path).await {
                continue;
            }
            let bytes = match self.files.read(&path).await {
                Ok(bytes) => bytes,
                Err(_) => continue,
            };
            match serde_json::from_slice::<InstanceRecord>(&bytes) {
                Ok(record) => records.push(record),
                Err(_) => continue,
            }
        }
        records.sort_by(|left, right| left.spec.id.cmp(&right.spec.id));
        Ok(records)
    }

    async fn get(&self, id: &str) -> CoreResult<Option<InstanceRecord>> {
        validate_id(id)?;
        let path = self.paths.instance_record(id);
        if !self.files.exists(&path).await {
            return Ok(None);
        }
        let bytes = self.files.read(&path).await?;
        let record: InstanceRecord = serde_json::from_slice(&bytes).map_err(|error| {
            CoreError::io(format!("instance record {} is damaged", path.display()))
                .with_details(error.to_string())
        })?;
        record.check_schema()?;
        Ok(Some(record))
    }

    async fn save(&self, record: &InstanceRecord) -> CoreResult<()> {
        validate_id(&record.spec.id)?;
        record.check_schema()?;
        let path = self.paths.instance_record(&record.spec.id);
        let bytes = serde_json::to_vec_pretty(record).map_err(|error| {
            CoreError::internal("the instance record could not be encoded")
                .with_details(error.to_string())
        })?;
        self.files.write_atomic(&path, &bytes).await
    }

    async fn delete(&self, id: &str) -> CoreResult<()> {
        validate_id(id)?;
        self.files.remove(&self.paths.instance_record(id)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mnelys_application::ports::DirEntry;
    use mnelys_domain::{InstanceRecord, InstanceSpec};
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

        async fn list(&self, path: &Path) -> CoreResult<Vec<DirEntry>> {
            let files = self.files.lock().expect("lock");
            let mut names = std::collections::BTreeSet::new();
            for key in files.keys() {
                if let Ok(relative) = key.strip_prefix(path) {
                    if let Some(first) = relative.components().next() {
                        names.insert(first.as_os_str().to_string_lossy().into_owned());
                    }
                }
            }
            Ok(names
                .into_iter()
                .map(|name| DirEntry {
                    path: path.join(&name),
                    name,
                    is_dir: true,
                })
                .collect())
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

    fn spec(id: &str) -> InstanceSpec {
        InstanceSpec {
            id: id.to_owned(),
            name: String::from("Test"),
            game_version: String::from("1.21.4"),
            loader: mnelys_domain::LoaderKind::Vanilla,
            loader_version: None,
        }
    }

    #[tokio::test]
    async fn records_round_trip_through_the_repository() {
        let files = Arc::new(MemoryFiles::default());
        let paths = GamePaths::new("/data");
        let repo = JsonInstanceRepository::new(files, paths);
        let record = InstanceRecord::new(spec("main"), 1, String::from("1.21.4"));
        repo.save(&record).await.expect("save");
        assert_eq!(repo.list().await.expect("list").len(), 1);
        assert_eq!(
            repo.get("main")
                .await
                .expect("get")
                .map(|found| found.spec.id),
            Some(String::from("main"))
        );
        repo.delete("main").await.expect("delete");
        assert!(repo.list().await.expect("list").is_empty());
    }

    #[tokio::test]
    async fn an_unsafe_id_is_rejected() {
        let repo =
            JsonInstanceRepository::new(Arc::new(MemoryFiles::default()), GamePaths::new("/data"));
        assert!(repo.get("../secrets").await.is_err());
    }
}
