//! Content installation: dependencies, files and portable instance manifests.
//!
//! Resolution is breadth-first over required dependencies with a hard cap. Optional and
//! embedded dependencies are not installed automatically: silently adding them would
//! change the user's mod set without consent.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use mnelys_domain::{
    ContentDependency, ContentKind, ContentProviderId, ContentSearchPage, ContentVersion,
    CoreError, CoreResult, DependencyKind, InstanceRecord, InstanceSpec,
};
use serde::{Deserialize, Serialize};

use crate::instances::{InstanceService, InstanceSummary};
use crate::paths::GamePaths;
use crate::ports::{
    fetch_bytes, Clock, ContentProvider, ContentSearchRequest, ContentVersionFilter, FileStore,
    HttpClient, InstanceRepository,
};
use crate::tasks::TaskReporter;
use crate::util::{hash_matches, parse_document};

const MAX_RESOLVED_PROJECTS: usize = 64;

/// The content providers this build knows about.
pub struct ContentRegistry {
    providers: Vec<Arc<dyn ContentProvider>>,
}

impl ContentRegistry {
    #[must_use]
    pub fn new(providers: Vec<Arc<dyn ContentProvider>>) -> Self {
        Self { providers }
    }

    pub fn get(&self, id: ContentProviderId) -> CoreResult<&Arc<dyn ContentProvider>> {
        self.providers
            .iter()
            .find(|provider| provider.id() == id)
            .ok_or_else(|| {
                CoreError::unavailable(format!(
                    "{} is not configured; add its credentials in settings",
                    id.as_str()
                ))
            })
    }

    #[must_use]
    pub fn available(&self) -> Vec<ContentProviderId> {
        self.providers
            .iter()
            .map(|provider| provider.id())
            .collect()
    }
}

/// One installed file, as the instance manifest records it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledContent {
    pub provider: ContentProviderId,
    pub project_id: String,
    pub version_id: String,
    pub name: String,
    pub filename: String,
    pub kind: ContentKind,
    pub installed_at_ms: u64,
}

/// The per-instance content manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentManifest {
    pub schema_version: u32,
    #[serde(default)]
    pub entries: Vec<InstalledContent>,
}

impl Default for ContentManifest {
    fn default() -> Self {
        Self {
            schema_version: 1,
            entries: Vec::new(),
        }
    }
}

/// A portable instance description. No tokens, no absolute paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceExport {
    pub schema_version: u32,
    pub spec: InstanceSpec,
    #[serde(default)]
    pub content: Vec<InstalledContent>,
}

/// What one content install did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentInstallReport {
    pub projects: Vec<String>,
    pub files: usize,
    pub bytes: u64,
}

/// The saved export, including where it landed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedInstance {
    pub path: String,
    pub json: String,
}

/// Searches, resolves and installs content.
pub struct ContentService {
    registry: ContentRegistry,
    client: Arc<dyn HttpClient>,
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    instances: Arc<InstanceService>,
    repo: Arc<dyn InstanceRepository>,
    clock: Arc<dyn Clock>,
}

impl ContentService {
    #[must_use]
    pub fn new(
        registry: ContentRegistry,
        client: Arc<dyn HttpClient>,
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        instances: Arc<InstanceService>,
        repo: Arc<dyn InstanceRepository>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            registry,
            client,
            files,
            paths,
            instances,
            repo,
            clock,
        }
    }

    #[must_use]
    pub fn available_providers(&self) -> Vec<ContentProviderId> {
        self.registry.available()
    }

    pub async fn search(
        &self,
        provider: ContentProviderId,
        mut request: ContentSearchRequest,
    ) -> CoreResult<ContentSearchPage> {
        if request.query.chars().count() > 128 {
            return Err(CoreError::invalid_request("search text is too long"));
        }
        request.limit = request.limit.clamp(1, 50);
        request.offset = request.offset.min(10_000);
        self.registry.get(provider)?.search(request).await
    }

    pub async fn versions(
        &self,
        provider: ContentProviderId,
        project_id: &str,
        game_version: Option<&str>,
        loader: Option<&str>,
    ) -> CoreResult<Vec<ContentVersion>> {
        let filter = ContentVersionFilter {
            game_version: game_version.map(str::to_owned),
            loader: loader.map(str::to_owned),
        };
        self.registry
            .get(provider)?
            .versions(project_id, &filter)
            .await
    }

    /// Resolves required dependencies and installs the files into an instance.
    pub async fn install(
        &self,
        instance_id: &str,
        provider_id: ContentProviderId,
        project_id: &str,
        version_id: Option<&str>,
        kind: ContentKind,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<ContentInstallReport> {
        let record = self.instances.record(instance_id).await?;
        let provider = self.registry.get(provider_id)?;
        let resolved = self
            .resolve(
                provider.as_ref(),
                &record,
                project_id,
                version_id,
                kind,
                reporter,
            )
            .await?;
        let game_dir = self.paths.instance_game_dir(&record.spec.id);
        let mut manifest = self.manifest(&record.spec.id).await?;
        let mut bytes = 0u64;

        reporter.progress(0, Some(resolved.len() as u64), "Downloading content");
        let mut projects = Vec::new();
        for (position, (version, content_kind)) in resolved.iter().enumerate() {
            reporter.check_cancelled()?;
            let file = version.primary_file().ok_or_else(|| {
                CoreError::unavailable(format!("{} has no downloadable file", version.name))
            })?;
            file.validate()?;
            let directory = game_dir.join(content_kind.directory());
            self.files.create_dir_all(&directory).await?;
            let destination = directory.join(&file.filename);
            let expected = file.sha1.as_deref();
            let already_there = {
                if self.files.exists(&destination).await {
                    match self.files.read(&destination).await {
                        Ok(existing) => hash_matches(&existing, expected),
                        Err(_) => false,
                    }
                } else {
                    false
                }
            };
            if !already_there {
                let data = fetch_bytes(self.client.as_ref(), &file.url).await?;
                if !hash_matches(&data, expected) {
                    return Err(CoreError::network(format!(
                        "{} did not match its published hash",
                        file.filename
                    )));
                }
                bytes += data.len() as u64;
                self.files.write_atomic(&destination, &data).await?;
            }

            if let Some(previous) = manifest
                .entries
                .iter()
                .find(|entry| entry.project_id == version.project_id)
            {
                if previous.filename != file.filename {
                    let old = game_dir
                        .join(previous.kind.directory())
                        .join(&previous.filename);
                    let _ = self.files.remove(&old).await;
                }
            }
            manifest
                .entries
                .retain(|entry| entry.project_id != version.project_id);
            manifest.entries.push(InstalledContent {
                provider: provider_id,
                project_id: version.project_id.clone(),
                version_id: version.id.clone(),
                name: version.name.clone(),
                filename: file.filename.clone(),
                kind: *content_kind,
                installed_at_ms: self.clock.now_ms(),
            });
            projects.push(version.project_id.clone());
            reporter.progress(
                position as u64 + 1,
                Some(resolved.len() as u64),
                &format!("Installed {}", file.filename),
            );
        }

        self.write_manifest(&record.spec.id, &manifest).await?;
        Ok(ContentInstallReport {
            projects,
            files: resolved.len(),
            bytes,
        })
    }

    async fn resolve(
        &self,
        provider: &dyn ContentProvider,
        record: &InstanceRecord,
        project_id: &str,
        version_id: Option<&str>,
        kind: ContentKind,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<Vec<(ContentVersion, ContentKind)>> {
        let filter = ContentVersionFilter {
            game_version: Some(record.spec.game_version.clone()),
            loader: Some(record.spec.loader.as_str().to_owned()),
        };
        let mut queue: VecDeque<(String, Option<String>, ContentKind)> = VecDeque::new();
        queue.push_back((project_id.to_owned(), version_id.map(str::to_owned), kind));
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut resolved = Vec::new();

        while let Some((project, version, content_kind)) = queue.pop_front() {
            reporter.check_cancelled()?;
            if !seen.insert(project.clone()) {
                continue;
            }
            if seen.len() > MAX_RESOLVED_PROJECTS {
                return Err(CoreError::invalid_request(
                    "this project pulls in too many dependencies",
                ));
            }
            let selected = match version {
                Some(version_id) => provider
                    .versions(&project, &ContentVersionFilter::default())
                    .await?
                    .into_iter()
                    .find(|candidate| candidate.id == version_id),
                None => provider
                    .versions(&project, &filter)
                    .await?
                    .into_iter()
                    .next(),
            };
            let selected = selected.ok_or_else(|| {
                CoreError::unavailable(format!(
                    "no compatible version of project `{project}` was found"
                ))
            })?;
            for dependency in &selected.dependencies {
                self.queue_dependency(dependency, &mut queue)?;
            }
            resolved.push((selected, content_kind));
        }
        Ok(resolved)
    }

    fn queue_dependency(
        &self,
        dependency: &ContentDependency,
        queue: &mut VecDeque<(String, Option<String>, ContentKind)>,
    ) -> CoreResult<()> {
        if dependency.kind != DependencyKind::Required {
            return Ok(());
        }
        let project = dependency
            .project_id
            .clone()
            .ok_or_else(|| CoreError::invalid_request("a required dependency has no project id"))?;
        queue.push_back((project, dependency.version_id.clone(), ContentKind::Mod));
        Ok(())
    }

    /// Returns the content manifest, or an empty one when the instance has none.
    pub async fn manifest(&self, instance_id: &str) -> CoreResult<ContentManifest> {
        let path = self.paths.instance_dir(instance_id).join("content.json");
        if !self.files.exists(&path).await {
            return Ok(ContentManifest::default());
        }
        let bytes = self.files.read(&path).await?;
        let manifest: ContentManifest = parse_document(&bytes, "the content manifest")?;
        if manifest.schema_version > 1 {
            return Err(CoreError::internal(
                "the content manifest was written by a newer Mnelys",
            ));
        }
        Ok(manifest)
    }

    async fn write_manifest(
        &self,
        instance_id: &str,
        manifest: &ContentManifest,
    ) -> CoreResult<()> {
        let bytes = serde_json::to_vec_pretty(manifest).map_err(|error| {
            CoreError::internal("could not encode the content manifest")
                .with_details(error.to_string())
        })?;
        self.files
            .write_atomic(
                &self.paths.instance_dir(instance_id).join("content.json"),
                &bytes,
            )
            .await
    }

    pub async fn remove(&self, instance_id: &str, project_id: &str) -> CoreResult<bool> {
        let record = self.instances.record(instance_id).await?;
        let mut manifest = self.manifest(instance_id).await?;
        let Some(entry) = manifest
            .entries
            .iter()
            .find(|entry| entry.project_id == project_id)
            .cloned()
        else {
            return Ok(false);
        };
        let path = self
            .paths
            .instance_game_dir(instance_id)
            .join(entry.kind.directory())
            .join(&entry.filename);
        let _ = self.files.remove(&path).await;
        manifest
            .entries
            .retain(|entry| entry.project_id != project_id);
        self.write_manifest(&record.spec.id, &manifest).await?;
        Ok(true)
    }

    /// Lists every project the instance records, for the instance detail page.
    pub async fn installed(&self, instance_id: &str) -> CoreResult<Vec<InstalledContent>> {
        Ok(self.manifest(instance_id).await?.entries)
    }

    /// Writes a portable manifest to the exports directory.
    pub async fn export(&self, instance_id: &str) -> CoreResult<ExportedInstance> {
        let record = self.instances.record(instance_id).await?;
        let manifest = self.manifest(instance_id).await?;
        let export = InstanceExport {
            schema_version: 1,
            spec: record.spec.clone(),
            content: manifest.entries,
        };
        let json = serde_json::to_string_pretty(&export).map_err(|error| {
            CoreError::internal("could not encode the instance export")
                .with_details(error.to_string())
        })?;
        self.files.create_dir_all(&self.paths.exports_dir()).await?;
        let file = self
            .paths
            .exports_dir()
            .join(format!("{}.mnelys.json", record.spec.id));
        self.files.write_atomic(&file, json.as_bytes()).await?;
        Ok(ExportedInstance {
            path: file.to_string_lossy().into_owned(),
            json,
        })
    }

    /// Imports a portable manifest, allocating a fresh id when it collides.
    pub async fn import(&self, json: &str) -> CoreResult<InstanceSummary> {
        let export: InstanceExport = serde_json::from_str(json).map_err(|error| {
            CoreError::invalid_request("the import file is not a valid instance export")
                .with_details(error.to_string())
        })?;
        if export.schema_version > 1 {
            return Err(CoreError::invalid_request(
                "the import file was written by a newer Mnelys",
            ));
        }
        let existing: BTreeSet<String> = self
            .repo
            .list()
            .await?
            .into_iter()
            .map(|record| record.spec.id)
            .collect();
        let mut spec = export.spec;
        spec.id = unique_id(&spec.id, &existing);
        let summary = self.instances.create(spec.clone()).await?;
        let manifest = ContentManifest {
            schema_version: 1,
            entries: export.content,
        };
        self.write_manifest(&spec.id, &manifest).await?;
        Ok(summary)
    }
}

fn unique_id(base: &str, existing: &BTreeSet<String>) -> String {
    if !existing.contains(base) {
        return base.to_owned();
    }
    let stem: String = base.chars().take(40).collect();
    for suffix in 2..1000 {
        let candidate = format!("{stem}-{suffix}");
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    format!("{stem}-{}", uuid::Uuid::new_v4())
}

/// Stable map of project id to its installed file, for diffing exports.
#[must_use]
pub fn by_project(entries: &[InstalledContent]) -> BTreeMap<&str, &InstalledContent> {
    entries
        .iter()
        .map(|entry| (entry.project_id.as_str(), entry))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_ids_grow_without_colliding() {
        let mut existing = BTreeSet::new();
        existing.insert(String::from("main"));
        assert_eq!(unique_id("main", &existing), "main-2");
        existing.insert(String::from("main-2"));
        assert_eq!(unique_id("main", &existing), "main-3");
    }

    #[test]
    fn a_manifest_is_keyed_by_project() {
        let entry = InstalledContent {
            provider: ContentProviderId::Modrinth,
            project_id: String::from("p"),
            version_id: String::from("v"),
            name: String::from("Project"),
            filename: String::from("p.jar"),
            kind: ContentKind::Mod,
            installed_at_ms: 1,
        };
        let entries = vec![entry];
        assert_eq!(by_project(&entries)["p"].version_id, "v");
    }
}
