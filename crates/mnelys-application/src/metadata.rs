//! Minecraft metadata: the version manifest, version documents and asset indexes.
//!
//! Everything is cached on disk, so an install that already has metadata works without
//! the network. A failed refresh falls back to the cached manifest instead of turning a
//! working launcher into an offline one.

use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

use mnelys_domain::{
    AssetIndexDocument, CoreError, CoreResult, InstanceSpec, VersionDocument, VersionManifest,
};

use crate::mirrors::MirrorPolicy;
use crate::paths::GamePaths;
use crate::ports::{fetch_bytes, Clock, FileStore, HttpClient, LoaderProvider};
use crate::util::{parse_document, validate_version_id};

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";
const MANIFEST_TTL_MS: u64 = 10 * 60 * 1000;

/// Reads, caches and resolves version metadata.
pub struct MetadataService {
    client: Arc<dyn HttpClient>,
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    clock: Arc<dyn Clock>,
    providers: Arc<dyn LoaderProvider>,
    mirrors: MirrorPolicy,
    manifest: RwLock<Option<(u64, VersionManifest)>>,
}

impl MetadataService {
    #[must_use]
    pub fn new(
        client: Arc<dyn HttpClient>,
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        clock: Arc<dyn Clock>,
        providers: Arc<dyn LoaderProvider>,
        mirrors: MirrorPolicy,
    ) -> Self {
        Self {
            client,
            files,
            paths,
            clock,
            providers,
            mirrors,
            manifest: RwLock::new(None),
        }
    }

    /// The version manifest, refreshed at most every ten minutes.
    pub async fn manifest(&self) -> CoreResult<VersionManifest> {
        let now = self.clock.now_ms();
        if let Some((cached_at, manifest)) = self
            .manifest
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .as_ref()
        {
            if now.saturating_sub(*cached_at) < MANIFEST_TTL_MS {
                return Ok(manifest.clone());
            }
        }

        self.mirrors.ensure(self.client.as_ref()).await;
        let url = self.mirrors.rewrite(MANIFEST_URL);
        match fetch_bytes(self.client.as_ref(), &url).await {
            Ok(bytes) => {
                let manifest: VersionManifest = parse_document(&bytes, "the version manifest")?;
                let path = self.paths.meta_dir().join("version_manifest_v2.json");
                self.files.write_atomic(&path, &bytes).await?;
                *self
                    .manifest
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner) =
                    Some((now, manifest.clone()));
                Ok(manifest)
            }
            Err(error) => match self.cached_manifest().await {
                Some(manifest) => Ok(manifest),
                None => Err(error.with_details("no cached manifest is available")),
            },
        }
    }

    async fn cached_manifest(&self) -> Option<VersionManifest> {
        let path = self.paths.meta_dir().join("version_manifest_v2.json");
        if !self.files.exists(&path).await {
            return None;
        }
        let bytes = self.files.read(&path).await.ok()?;
        parse_document(&bytes, "the cached version manifest").ok()
    }

    /// One version document, read from disk when possible.
    pub async fn version(&self, id: &str) -> CoreResult<VersionDocument> {
        validate_version_id(id)?;
        let path = self
            .paths
            .meta_dir()
            .join("versions")
            .join(format!("{id}.json"));
        if self.files.exists(&path).await {
            let bytes = self.files.read(&path).await?;
            return parse_document(&bytes, &format!("the cached document for {id}"));
        }

        let manifest = self.manifest().await?;
        let entry = manifest
            .versions
            .iter()
            .find(|entry| entry.id == id)
            .ok_or_else(|| {
                CoreError::unavailable(format!("Minecraft {id} is not in the manifest"))
            })?;
        let bytes = fetch_bytes(self.client.as_ref(), &self.mirrors.rewrite(&entry.url)).await?;
        let document: VersionDocument =
            parse_document(&bytes, &format!("the version document for {id}"))?;
        if document.id != id {
            return Err(CoreError::network(format!(
                "the document for {id} identifies itself as {}",
                document.id
            )));
        }
        self.files.write_atomic(&path, &bytes).await?;
        Ok(document)
    }

    /// Resolves an instance spec to the executable version document.
    ///
    /// Installer-based loaders are rejected here: they need Java and the Vanilla files
    /// first, which only the install service can provide.
    pub async fn resolve(&self, spec: &InstanceSpec) -> CoreResult<VersionDocument> {
        let base = self.version(&spec.game_version).await?;
        if spec.loader == mnelys_domain::LoaderKind::Vanilla {
            return Ok(base);
        }
        if spec.loader.uses_installer() {
            return Err(CoreError::not_implemented(format!(
                "{} is installed by its own installer",
                spec.loader.as_str()
            )));
        }
        let loader_version = spec.loader_version.as_deref().ok_or_else(|| {
            CoreError::invalid_request("a loader version is required for this instance")
        })?;
        let profile = self
            .providers
            .profile(spec.loader, &spec.game_version, loader_version)
            .await?;
        let mut merged = base.merge_profile(&profile);
        merged.id = spec.version_id();
        Ok(merged)
    }

    /// The asset index objects a version needs.
    pub async fn asset_index(&self, document: &VersionDocument) -> CoreResult<AssetIndexDocument> {
        let id = document.asset_index_id();
        validate_version_id(&id)?;
        let path = self.paths.asset_index_file(&id);
        if self.files.exists(&path).await {
            let bytes = self.files.read(&path).await?;
            return parse_document(&bytes, &format!("the cached asset index {id}"));
        }
        let Some(index) = &document.asset_index else {
            return Ok(AssetIndexDocument {
                objects: BTreeMap::new(),
            });
        };
        let bytes = fetch_bytes(self.client.as_ref(), &self.mirrors.rewrite(&index.url)).await?;
        let parsed: AssetIndexDocument = parse_document(&bytes, &format!("the asset index {id}"))?;
        self.files.write_atomic(&path, &bytes).await?;
        Ok(parsed)
    }

    /// Writes the merged document that install and launch read back.
    pub async fn save_version(&self, document: &VersionDocument) -> CoreResult<()> {
        validate_version_id(&document.id)?;
        let bytes = serde_json::to_vec_pretty(document).map_err(|error| {
            CoreError::internal("could not encode the version document")
                .with_details(error.to_string())
        })?;
        self.files
            .write_atomic(&self.paths.version_json(&document.id), &bytes)
            .await
    }

    /// Reads an installed version document.
    pub async fn installed_document(&self, version_id: &str) -> CoreResult<VersionDocument> {
        validate_version_id(version_id)?;
        let path = self.paths.version_json(version_id);
        if !self.files.exists(&path).await {
            return Err(CoreError::unavailable(format!(
                "version {version_id} is not installed"
            )));
        }
        let bytes = self.files.read(&path).await?;
        parse_document(&bytes, &format!("the installed document for {version_id}"))
    }
}
