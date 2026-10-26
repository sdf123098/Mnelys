//! Installing a version: plan every file, download what is missing, verify every hash.
//!
//! The installer is deliberately sequential. Minecraft installs are dominated by
//! network latency and a few large files, and a sequential loop gives honest progress,
//! straightforward cancellation and no thundering herd against a CDN.

use std::sync::Arc;

use mnelys_domain::{
    AssetIndexDocument, CoreError, CoreResult, DownloadItem, DownloadKind, DownloadPlan,
    InstanceRecord, Library, LoaderKind, MavenCoordinate, Platform, VersionDocument,
};

use crate::instances::InstanceService;
use crate::metadata::MetadataService;
use crate::mirrors::MirrorPolicy;
use crate::paths::GamePaths;
use crate::ports::{
    fetch_bytes, Clock, FileStore, HttpClient, JavaProvider, LoaderInstallContext, LoaderProvider,
};
use crate::tasks::TaskReporter;
use crate::util::{hash_matches, join_url, url_file_name};

/// What one install did.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallReport {
    pub version_id: String,
    pub files: usize,
    pub downloaded: usize,
    pub skipped: usize,
    pub bytes: u64,
}

/// Builds and executes install plans.
pub struct InstallService {
    client: Arc<dyn HttpClient>,
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    metadata: Arc<MetadataService>,
    instances: Arc<InstanceService>,
    providers: Arc<dyn LoaderProvider>,
    java: Arc<dyn JavaProvider>,
    mirrors: MirrorPolicy,
    clock: Arc<dyn Clock>,
}

impl InstallService {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        client: Arc<dyn HttpClient>,
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        metadata: Arc<MetadataService>,
        instances: Arc<InstanceService>,
        providers: Arc<dyn LoaderProvider>,
        java: Arc<dyn JavaProvider>,
        mirrors: MirrorPolicy,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            client,
            files,
            paths,
            metadata,
            instances,
            providers,
            java,
            mirrors,
            clock,
        }
    }

    /// Installs everything an instance needs to start.
    pub async fn install(
        &self,
        instance_id: &str,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<InstallReport> {
        let record = self.instances.record(instance_id).await?;
        reporter.progress(0, None, "Resolving version metadata");
        let document = self.resolve_document(&record, reporter).await?;
        let report = self
            .install_document(&document, &record.spec.id, reporter)
            .await?;
        if record.version_id != document.id {
            let mut updated = record;
            updated.version_id = document.id.clone();
            self.instances.save_record(&updated).await?;
        }
        Ok(report)
    }

    async fn resolve_document(
        &self,
        record: &InstanceRecord,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<VersionDocument> {
        let spec = &record.spec;
        let base = self.metadata.version(&spec.game_version).await?;
        match spec.loader {
            LoaderKind::Vanilla => Ok(base),
            loader if loader.uses_metadata_profile() => self.metadata.resolve(spec).await,
            loader => {
                // The installer needs the Vanilla jar and libraries on disk first.
                reporter.progress(0, None, "Installing the Vanilla base");
                self.install_document(&base, &spec.id, reporter).await?;

                let loader_version = spec.loader_version.clone().ok_or_else(|| {
                    CoreError::invalid_request("a loader version is required for this instance")
                })?;
                let java_path = self.pick_java(record, base.required_java_major()).await?;
                let context = LoaderInstallContext {
                    game_version: spec.game_version.clone(),
                    loader_version: loader_version.clone(),
                    java_path: std::path::PathBuf::from(java_path),
                    game_paths: self.paths.clone(),
                    vanilla_jar: self.paths.version_jar(&spec.game_version),
                    vanilla_version_json: self.paths.version_json(&spec.game_version),
                };
                reporter.progress(0, None, "Running the loader installer");
                let installed = self
                    .providers
                    .install(loader, &context, reporter)
                    .await?
                    .ok_or_else(|| {
                        CoreError::internal(format!(
                            "{} did not produce a version document",
                            loader.as_str()
                        ))
                    })?;
                let mut merged = base.merge_profile(&installed);
                merged.id = spec.version_id();
                Ok(merged)
            }
        }
    }

    async fn pick_java(&self, record: &InstanceRecord, required: u32) -> CoreResult<String> {
        if let Some(path) = &record.java_path {
            return Ok(path.clone());
        }
        let mut runtimes = self.java.discover().await?;
        runtimes.retain(|runtime| runtime.satisfies(required));
        runtimes.sort_by_key(|runtime| std::cmp::Reverse(runtime.major));
        runtimes
            .into_iter()
            .next()
            .map(|runtime| runtime.path)
            .ok_or_else(|| {
                CoreError::unavailable(format!(
                    "Java {required} or newer is required; install a runtime first"
                ))
            })
    }

    /// Downloads and installs one resolved document.
    pub async fn install_document(
        &self,
        document: &VersionDocument,
        instance_id: &str,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<InstallReport> {
        let version_id = document.id.clone();
        crate::util::validate_version_id(&version_id)?;
        reporter.progress(0, None, "Planning downloads");
        let planned = self.build_plan(document).await?;
        let total = planned.plan.len() as u64;
        let mut downloaded = 0usize;
        let mut skipped = 0usize;
        let mut bytes = 0u64;

        for (index, item) in planned.plan.items().iter().enumerate() {
            reporter.check_cancelled()?;
            let destination = self.paths.root().join(&item.path);
            let name = url_file_name(&item.url).unwrap_or(&item.path);
            reporter.progress(
                index as u64,
                Some(total),
                &format!("Downloading {name} ({} of {total})", index + 1),
            );
            if self.is_current(&destination, item).await {
                skipped += 1;
                continue;
            }
            bytes += self.download(item, &destination, reporter).await?;
            downloaded += 1;
        }

        if !planned.plan.of_kind(DownloadKind::Native).is_empty() {
            reporter.progress(0, None, "Extracting native libraries");
            let natives = self.paths.version_natives(&version_id);
            let _ = self.files.remove_dir_all(&natives).await;
            self.files.create_dir_all(&natives).await?;
            let excluded = vec![String::from("META-INF/")];
            for item in planned.plan.of_kind(DownloadKind::Native) {
                let archive = self.paths.root().join(&item.path);
                self.files
                    .extract_zip(&archive, &natives, &excluded)
                    .await?;
            }
        }

        self.materialize_virtual_assets(document, instance_id, reporter)
            .await?;

        reporter.progress(total, Some(total), "Writing version metadata");
        self.metadata.save_version(document).await?;
        let marker = serde_json::json!({
            "versionId": version_id,
            "installedAtMs": self.clock.now_ms(),
            "files": planned.plan.len(),
        });
        self.files
            .write_atomic(
                &self.paths.installed_marker(&version_id),
                serde_json::to_vec_pretty(&marker)
                    .unwrap_or_default()
                    .as_slice(),
            )
            .await?;

        Ok(InstallReport {
            version_id,
            files: planned.plan.len(),
            downloaded,
            skipped,
            bytes,
        })
    }

    /// Builds the full download plan for a version.
    pub async fn build_plan(&self, document: &VersionDocument) -> CoreResult<PlannedInstall> {
        let version_id = &document.id;
        let platform = Platform::current();
        let mut items: Vec<DownloadItem> = Vec::new();

        if let Some(client) = document.client_download() {
            items.push(DownloadItem::new(
                client.url.clone(),
                format!("versions/{version_id}/{version_id}.jar"),
                DownloadKind::Client,
                client.sha1.clone(),
                client.size,
            ));
        }

        for library in &document.libraries {
            collect_library(library, platform, &mut items);
        }

        if let Some(logging) = document
            .logging
            .as_ref()
            .and_then(|logging| logging.client.as_ref())
        {
            let name = logging
                .file
                .id
                .clone()
                .or_else(|| url_file_name(&logging.file.url).map(str::to_owned))
                .unwrap_or_else(|| String::from("client.xml"));
            items.push(DownloadItem::new(
                logging.file.url.clone(),
                format!("assets/log_configs/{name}"),
                DownloadKind::Logging,
                logging.file.sha1.clone(),
                logging.file.size,
            ));
        }

        let asset_index = self.metadata.asset_index(document).await?;
        if let Some(index) = &document.asset_index {
            let id = document.asset_index_id();
            items.push(DownloadItem::new(
                index.url.clone(),
                format!("assets/indexes/{id}.json"),
                DownloadKind::Asset,
                index.sha1.clone(),
                index.size,
            ));
        }
        for (name, object) in &asset_index.objects {
            let prefix = object.hash.get(0..2).ok_or_else(|| {
                CoreError::network(format!("asset `{name}` has a malformed hash"))
            })?;
            items.push(DownloadItem::new(
                format!(
                    "https://resources.download.minecraft.net/{prefix}/{}",
                    object.hash
                ),
                format!("assets/objects/{prefix}/{}", object.hash),
                DownloadKind::Asset,
                Some(object.hash.clone()),
                Some(object.size),
            ));
        }

        let plan = DownloadPlan::new(items)?;
        Ok(PlannedInstall { plan, asset_index })
    }

    async fn is_current(&self, path: &std::path::Path, item: &DownloadItem) -> bool {
        if !self.files.exists(path).await {
            return false;
        }
        if let Some(size) = item.size {
            if self.files.len(path).await != Some(size) {
                return false;
            }
        }
        match &item.sha1 {
            Some(expected) => match self.files.read(path).await {
                Ok(bytes) => hash_matches(&bytes, Some(expected)),
                Err(_) => false,
            },
            None => true,
        }
    }

    async fn download(
        &self,
        item: &DownloadItem,
        destination: &std::path::Path,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<u64> {
        let rewritten = self.mirrors.rewrite(&item.url);
        let mut last_error: Option<CoreError> = None;
        for attempt in 1..=3u32 {
            reporter.check_cancelled()?;
            // The last attempt goes straight to the source, so a broken mirror degrades
            // into a slow download instead of a hard failure.
            let url = if attempt == 3 { &item.url } else { &rewritten };
            match fetch_bytes(self.client.as_ref(), url).await {
                Ok(bytes) => {
                    if !hash_matches(&bytes, item.sha1.as_deref()) {
                        last_error = Some(
                            CoreError::network(format!(
                                "{} did not match its published SHA-1",
                                item.path
                            ))
                            .with_details(format!("attempt {attempt}")),
                        );
                    } else {
                        if let Some(parent) = destination.parent() {
                            self.files.create_dir_all(parent).await?;
                        }
                        let length = bytes.len() as u64;
                        self.files.write_atomic(destination, &bytes).await?;
                        return Ok(length);
                    }
                }
                Err(error) => last_error = Some(error),
            }
            tokio::time::sleep(std::time::Duration::from_millis(400 * u64::from(attempt))).await;
        }
        Err(last_error
            .unwrap_or_else(|| CoreError::network(format!("could not download {}", item.path)))
            .with_details(format!("{} failed after 3 attempts", item.path)))
    }

    async fn materialize_virtual_assets(
        &self,
        document: &VersionDocument,
        instance_id: &str,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<()> {
        let Some(kind) = document.assets.as_deref() else {
            return Ok(());
        };
        if kind != "legacy" && kind != "pre-1.6" {
            return Ok(());
        }
        let index = self.metadata.asset_index(document).await?;
        if index.objects.is_empty() {
            return Ok(());
        }
        let target = if kind == "legacy" {
            self.paths.virtual_assets_dir(&document.asset_index_id())
        } else {
            self.paths.instance_game_dir(instance_id).join("resources")
        };
        reporter.progress(
            0,
            Some(index.objects.len() as u64),
            "Unpacking legacy assets",
        );
        for (position, (name, object)) in index.objects.iter().enumerate() {
            reporter.check_cancelled()?;
            let prefix = object
                .hash
                .get(0..2)
                .ok_or_else(|| CoreError::network("an asset hash is malformed"))?;
            let source = self
                .paths
                .asset_objects_dir()
                .join(prefix)
                .join(&object.hash);
            if !self.files.exists(&source).await {
                continue;
            }
            let destination = target.join(name);
            if !self.files.exists(&destination).await {
                self.files.copy(&source, &destination).await?;
            }
            if position % 64 == 0 {
                reporter.progress(
                    position as u64,
                    Some(index.objects.len() as u64),
                    "Unpacking legacy assets",
                );
            }
        }
        Ok(())
    }
}

/// A download plan plus the asset index used to build it.
#[derive(Debug, Clone)]
pub struct PlannedInstall {
    pub plan: DownloadPlan,
    pub asset_index: AssetIndexDocument,
}

fn collect_library(library: &Library, platform: Platform, items: &mut Vec<DownloadItem>) {
    if !mnelys_domain::rules_allow(&library.rules, platform) {
        return;
    }
    let coordinate = MavenCoordinate::parse(&library.name).ok();
    let downloads = library.downloads.as_ref();
    let mut pushed = false;

    if let Some(artifact) = downloads.and_then(|downloads| downloads.artifact.as_ref()) {
        let path = artifact
            .path
            .clone()
            .or_else(|| coordinate.as_ref().map(MavenCoordinate::repository_path));
        if let Some(path) = path {
            items.push(DownloadItem::new(
                artifact.url.clone(),
                format!("libraries/{path}"),
                DownloadKind::Library,
                artifact.sha1.clone(),
                artifact.size,
            ));
            pushed = true;
        }
    }

    if let (Some(natives), Some(classifiers)) = (
        library.natives.as_ref(),
        downloads.and_then(|downloads| downloads.classifiers.as_ref()),
    ) {
        if let Some(classifier) = natives.get(platform.metadata_name()) {
            let classifier = classifier.replace("${arch}", "64");
            if let Some(download) = classifiers.get(&classifier) {
                if let Some(path) = &download.path {
                    items.push(DownloadItem::new(
                        download.url.clone(),
                        format!("libraries/{path}"),
                        DownloadKind::Native,
                        download.sha1.clone(),
                        download.size,
                    ));
                    pushed = true;
                }
            }
        }
    }

    if pushed {
        return;
    }

    if let Some(coordinate) = coordinate {
        let path = coordinate.repository_path();
        let url = library.url.as_ref().map_or_else(
            || format!("https://libraries.minecraft.net/{path}"),
            |base| join_url(base, &path),
        );
        items.push(DownloadItem::new(
            url,
            format!("libraries/{path}"),
            DownloadKind::Library,
            None,
            None,
        ));
    }
}
