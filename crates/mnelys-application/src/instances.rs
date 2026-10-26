//! Instance lifecycle: create, list, edit and delete.
//!
//! Creation never touches the network. It validates the spec and writes a record, so
//! the instance page stays responsive and an install is always an explicit action.

use std::sync::Arc;

use mnelys_domain::{CoreError, CoreResult, InstanceRecord, InstanceSpec, LoaderKind};
use serde::{Deserialize, Serialize};

use crate::paths::GamePaths;
use crate::ports::{Clock, FileStore, InstanceRepository};

/// A row on the instances page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSummary {
    pub id: String,
    pub name: String,
    pub game_version: String,
    pub loader: LoaderKind,
    #[serde(default)]
    pub loader_version: Option<String>,
    pub version_id: String,
    pub created_at_ms: u64,
    #[serde(default)]
    pub last_launched_ms: Option<u64>,
    #[serde(default)]
    pub java_path: Option<String>,
    pub installed: bool,
    pub directory: String,
}

/// Fields an edit may change.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInstance {
    #[serde(default)]
    pub name: Option<String>,
    /// Empty string clears the override and returns to automatic Java selection.
    #[serde(default)]
    pub java_path: Option<String>,
}

/// Coordinates the instance repository with the game directory layout.
pub struct InstanceService {
    repo: Arc<dyn InstanceRepository>,
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    clock: Arc<dyn Clock>,
}

impl InstanceService {
    #[must_use]
    pub fn new(
        repo: Arc<dyn InstanceRepository>,
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            repo,
            files,
            paths,
            clock,
        }
    }

    pub async fn list(&self) -> CoreResult<Vec<InstanceSummary>> {
        let mut records = self.repo.list().await?;
        records.sort_by(|left, right| {
            right
                .last_launched_ms
                .unwrap_or(0)
                .cmp(&left.last_launched_ms.unwrap_or(0))
                .then_with(|| left.spec.name.cmp(&right.spec.name))
        });
        let mut summaries = Vec::with_capacity(records.len());
        for record in &records {
            summaries.push(self.summarize(record).await);
        }
        Ok(summaries)
    }

    pub async fn get(&self, id: &str) -> CoreResult<InstanceSummary> {
        let record = self.record(id).await?;
        Ok(self.summarize(&record).await)
    }

    pub async fn record(&self, id: &str) -> CoreResult<InstanceRecord> {
        self.repo
            .get(id)
            .await?
            .ok_or_else(|| CoreError::unavailable(format!("instance `{id}` does not exist")))
    }

    pub async fn create(&self, spec: InstanceSpec) -> CoreResult<InstanceSummary> {
        spec.validate()?;
        if self.repo.get(&spec.id).await?.is_some() {
            return Err(CoreError::conflict(format!(
                "instance `{}` already exists",
                spec.id
            )));
        }
        let now = self.clock.now_ms();
        let record = InstanceRecord::new(spec, now, String::new());
        let mut record = record;
        record.version_id = record.spec.version_id();
        self.prepare_directories(&record.spec.id).await?;
        self.repo.save(&record).await?;
        Ok(self.summarize(&record).await)
    }

    async fn prepare_directories(&self, id: &str) -> CoreResult<()> {
        for path in [
            self.paths.instance_dir(id),
            self.paths.instance_game_dir(id),
            self.paths.instance_game_dir(id).join("mods"),
            self.paths.instance_game_dir(id).join("resourcepacks"),
            self.paths.instance_game_dir(id).join("shaderpacks"),
            self.paths.instance_game_dir(id).join("saves"),
        ] {
            self.files.create_dir_all(&path).await?;
        }
        Ok(())
    }

    pub async fn update(&self, id: &str, update: UpdateInstance) -> CoreResult<InstanceSummary> {
        let mut record = self.record(id).await?;
        if let Some(name) = update.name {
            let mut spec = record.spec.clone();
            spec.name = name;
            spec.validate()?;
            record.spec.name = spec.name;
        }
        if let Some(java_path) = update.java_path {
            let trimmed = java_path.trim();
            if trimmed.is_empty() {
                record.java_path = None;
            } else {
                let path = std::path::Path::new(trimmed);
                if !self.files.exists(path).await {
                    return Err(CoreError::invalid_request(
                        "the selected Java executable does not exist",
                    ));
                }
                record.java_path = Some(trimmed.to_owned());
            }
        }
        self.repo.save(&record).await?;
        Ok(self.summarize(&record).await)
    }

    pub async fn delete(&self, id: &str) -> CoreResult<()> {
        self.record(id).await?;
        self.repo.delete(id).await?;
        self.files
            .remove_dir_all(&self.paths.instance_dir(id))
            .await
    }

    pub async fn touch_launch(&self, id: &str) -> CoreResult<()> {
        let mut record = self.record(id).await?;
        record.last_launched_ms = Some(self.clock.now_ms());
        self.repo.save(&record).await
    }

    pub async fn set_java(&self, id: &str, java_path: Option<String>) -> CoreResult<()> {
        let mut record = self.record(id).await?;
        record.java_path = java_path;
        self.repo.save(&record).await
    }

    /// Saves a record the install service updated.
    pub async fn save_record(&self, record: &InstanceRecord) -> CoreResult<()> {
        self.repo.save(record).await
    }

    async fn summarize(&self, record: &InstanceRecord) -> InstanceSummary {
        let installed = self
            .files
            .exists(&self.paths.installed_marker(&record.version_id))
            .await
            && self
                .files
                .exists(&self.paths.version_json(&record.version_id))
                .await;
        InstanceSummary {
            id: record.spec.id.clone(),
            name: record.spec.name.clone(),
            game_version: record.spec.game_version.clone(),
            loader: record.spec.loader,
            loader_version: record.spec.loader_version.clone(),
            version_id: record.version_id.clone(),
            created_at_ms: record.created_at_ms,
            last_launched_ms: record.last_launched_ms,
            java_path: record.java_path.clone(),
            installed,
            directory: self
                .paths
                .instance_game_dir(&record.spec.id)
                .to_string_lossy()
                .into_owned(),
        }
    }
}
