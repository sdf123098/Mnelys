//! Forge and NeoForge installation.
//!
//! Both projects install through an installer jar instead of publishing a ready-made
//! profile. Mnelys reads `install_profile.json`, unpacks the bundled Maven artifacts,
//! downloads the installer libraries, then runs the client-side processors with the
//! Java runtime the user selected. Processor output is verified against the SHA-1 the
//! profile publishes before the profile is handed back to the install service.

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mnelys_application::ports::{fetch_bytes, HttpClient, LoaderInstallContext, LoaderProvider};
use mnelys_application::TaskReporter;
use mnelys_domain::{
    classpath_separator, CoreError, CoreResult, DownloadInfo, Library, LoaderKind,
    LoaderVersionInfo, MavenCoordinate, VersionDocument,
};
use serde::Deserialize;
use sha1::{Digest, Sha1};
use tokio::process::Command;

const FORGE_MAVEN: &str = "https://maven.minecraftforge.net/";
const NEOFORGE_MAVEN: &str = "https://maven.neoforged.net/releases/";
const FORGE_METADATA: &str =
    "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
const NEOFORGE_METADATA: &str =
    "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
const PROCESSOR_TIMEOUT: Duration = Duration::from_secs(300);

/// Installs Forge and NeoForge from their installer jars.
pub struct InstallerLoaderProvider {
    client: Arc<dyn HttpClient>,
}

impl InstallerLoaderProvider {
    #[must_use]
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self { client }
    }
}

fn maven_base(loader: LoaderKind) -> CoreResult<&'static str> {
    match loader {
        LoaderKind::Forge => Ok(FORGE_MAVEN),
        LoaderKind::NeoForge => Ok(NEOFORGE_MAVEN),
        other => Err(CoreError::not_implemented(format!(
            "{} is not installed by an installer",
            other.as_str()
        ))),
    }
}

/// The Maven version of the installer artifact.
fn artifact_version(loader: LoaderKind, game_version: &str, loader_version: &str) -> String {
    match loader {
        LoaderKind::Forge => {
            let prefix = format!("{game_version}-");
            if loader_version.starts_with(&prefix) {
                loader_version.to_owned()
            } else {
                format!("{game_version}-{loader_version}")
            }
        }
        _ => loader_version.to_owned(),
    }
}

fn installer_url(
    loader: LoaderKind,
    game_version: &str,
    loader_version: &str,
) -> CoreResult<String> {
    let version = artifact_version(loader, game_version, loader_version);
    let url = match loader {
        LoaderKind::Forge => {
            format!("{FORGE_MAVEN}net/minecraftforge/forge/{version}/forge-{version}-installer.jar")
        }
        LoaderKind::NeoForge => format!(
            "{NEOFORGE_MAVEN}net/neoforged/neoforge/{version}/neoforge-{version}-installer.jar"
        ),
        other => {
            return Err(CoreError::not_implemented(format!(
                "{} is not installed by an installer",
                other.as_str()
            )))
        }
    };
    Ok(url)
}

/// NeoForge numbers releases after the Minecraft version with the leading `1.` dropped.
///
/// `1.21.4` becomes `21.4.`, `1.21` becomes `21.0.` and the year-based versions
/// introduced later (`26.3`) keep their own prefix.
fn neoforge_prefix(game_version: &str) -> Option<String> {
    let parts: Vec<&str> = game_version.split('.').collect();
    match parts.as_slice() {
        ["1", minor, patch, ..] => Some(format!("{minor}.{patch}.")),
        ["1", minor] => Some(format!("{minor}.0.")),
        [year, minor, patch, ..] => Some(format!("{year}.{minor}.{patch}.")),
        [year, minor] => Some(format!("{year}.{minor}.")),
        _ => None,
    }
}

fn parse_metadata_versions(xml: &str) -> Vec<String> {
    let mut versions = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<version>") {
        rest = &rest[start + "<version>".len()..];
        let Some(end) = rest.find("</version>") else {
            break;
        };
        let version = rest[..end].trim();
        if !version.is_empty() {
            versions.push(version.to_owned());
        }
        rest = &rest[end + "</version>".len()..];
    }
    versions
}

fn version_key(version: &str) -> Vec<u64> {
    version
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .collect()
}

#[async_trait]
impl LoaderProvider for InstallerLoaderProvider {
    async fn versions(
        &self,
        loader: LoaderKind,
        game_version: &str,
    ) -> CoreResult<Vec<LoaderVersionInfo>> {
        let url = match loader {
            LoaderKind::Forge => FORGE_METADATA,
            LoaderKind::NeoForge => NEOFORGE_METADATA,
            other => {
                return Err(CoreError::not_implemented(format!(
                    "{} does not use installer metadata",
                    other.as_str()
                )))
            }
        };
        let bytes = fetch_bytes(self.client.as_ref(), url).await?;
        let xml = String::from_utf8_lossy(&bytes);
        let mut versions: Vec<String> = match loader {
            LoaderKind::Forge => {
                let prefix = format!("{game_version}-");
                parse_metadata_versions(&xml)
                    .into_iter()
                    .filter_map(|version| version.strip_prefix(&prefix).map(str::to_owned))
                    .collect()
            }
            LoaderKind::NeoForge => {
                let prefix = neoforge_prefix(game_version).ok_or_else(|| {
                    CoreError::invalid_request(format!(
                        "`{game_version}` is not a Minecraft version NeoForge can name"
                    ))
                })?;
                parse_metadata_versions(&xml)
                    .into_iter()
                    .filter(|version| version.starts_with(&prefix))
                    .collect()
            }
            _ => unreachable!("checked above"),
        };
        versions.sort_by_key(|version| std::cmp::Reverse(version_key(version)));
        versions.dedup();
        Ok(versions
            .into_iter()
            .map(|version| LoaderVersionInfo {
                version,
                game_version: game_version.to_owned(),
                loader,
                stable: true,
            })
            .collect())
    }

    async fn profile(
        &self,
        loader: LoaderKind,
        _game_version: &str,
        _loader_version: &str,
    ) -> CoreResult<VersionDocument> {
        Err(CoreError::not_implemented(format!(
            "{} publishes its profile inside the installer",
            loader.as_str()
        )))
    }

    async fn install(
        &self,
        loader: LoaderKind,
        context: &LoaderInstallContext,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<Option<VersionDocument>> {
        match loader {
            LoaderKind::Forge | LoaderKind::NeoForge => self
                .install_inner(loader, context, reporter)
                .await
                .map(Some),
            _ => Ok(None),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct InstallProfile {
    #[serde(default)]
    spec: u32,
    #[serde(default)]
    minecraft: Option<String>,
    #[serde(default)]
    data: BTreeMap<String, SideValue>,
    #[serde(default)]
    processors: Vec<ProcessorSpec>,
    #[serde(default)]
    libraries: Vec<Library>,
}

#[derive(Debug, Default, Deserialize)]
struct SideValue {
    #[serde(default)]
    client: String,
}

#[derive(Debug, Deserialize)]
struct ProcessorSpec {
    #[serde(default)]
    sides: Vec<String>,
    jar: String,
    #[serde(default)]
    classpath: Vec<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    outputs: BTreeMap<String, String>,
}

impl ProcessorSpec {
    fn runs_on_client(&self) -> bool {
        self.sides.is_empty() || self.sides.iter().any(|side| side == "client")
    }
}

impl InstallerLoaderProvider {
    async fn install_inner(
        &self,
        loader: LoaderKind,
        context: &LoaderInstallContext,
        reporter: &dyn TaskReporter,
    ) -> CoreResult<VersionDocument> {
        let url = installer_url(loader, &context.game_version, &context.loader_version)?;
        let cache = context.game_paths.cache_dir().join("installers");
        tokio::fs::create_dir_all(&cache).await.map_err(|error| {
            CoreError::io("could not create the installer cache").with_details(error.to_string())
        })?;
        let installer = cache.join(url.rsplit('/').next().unwrap_or("installer.jar"));
        if !installer.is_file() {
            reporter.progress(0, None, "Downloading the loader installer");
            let bytes = fetch_bytes(self.client.as_ref(), &url).await?;
            tokio::fs::write(&installer, &bytes)
                .await
                .map_err(|error| {
                    CoreError::io("could not save the loader installer")
                        .with_details(error.to_string())
                })?;
        }

        reporter.progress(10, Some(100), "Reading the installer profile");
        let (profile_bytes, version_bytes) = {
            let archive = installer.clone();
            tokio::task::spawn_blocking(move || read_installer_documents(&archive))
                .await
                .map_err(|error| {
                    CoreError::internal("the installer reader failed")
                        .with_details(error.to_string())
                })??
        };
        let profile: InstallProfile = serde_json::from_slice(&profile_bytes).map_err(|error| {
            CoreError::network("the loader installer profile is damaged")
                .with_details(error.to_string())
        })?;
        if profile.spec != 1 || profile.processors.is_empty() {
            return Err(CoreError::not_implemented(format!(
                "{} {} uses a legacy installer this build cannot run",
                loader.as_str(),
                context.loader_version
            )));
        }
        if let Some(expected) = profile.minecraft.as_deref() {
            if expected != context.game_version {
                return Err(CoreError::network(format!(
                    "the installer targets {expected}, not {}",
                    context.game_version
                )));
            }
        }
        let mut document: VersionDocument =
            serde_json::from_slice(&version_bytes).map_err(|error| {
                CoreError::network("the installer version profile is damaged")
                    .with_details(error.to_string())
            })?;
        document.inherits_from = Some(context.game_version.clone());

        let libraries = context.game_paths.libraries_dir();
        reporter.progress(15, Some(100), "Unpacking bundled libraries");
        {
            let archive = installer.clone();
            let destination = libraries.clone();
            tokio::task::spawn_blocking(move || extract_maven(&archive, &destination))
                .await
                .map_err(|error| {
                    CoreError::internal("the installer unpacker failed")
                        .with_details(error.to_string())
                })??;
        }

        let work = context
            .game_paths
            .cache_dir()
            .join("installer-work")
            .join(format!(
                "{}-{}",
                context.loader_version,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|elapsed| elapsed.as_nanos())
                    .unwrap_or(0)
            ));
        tokio::fs::create_dir_all(&work).await.map_err(|error| {
            CoreError::io("could not create the installer work directory")
                .with_details(error.to_string())
        })?;

        let index = library_index(&profile.libraries);
        let base = maven_base(loader)?;

        reporter.progress(20, Some(100), "Downloading installer libraries");
        for library in &profile.libraries {
            ensure_artifact(
                &index,
                &library.name,
                &libraries,
                self.client.as_ref(),
                base,
            )
            .await?;
        }
        for processor in &profile.processors {
            if !processor.runs_on_client() {
                continue;
            }
            ensure_artifact(
                &index,
                &processor.jar,
                &libraries,
                self.client.as_ref(),
                base,
            )
            .await?;
            for entry in &processor.classpath {
                ensure_artifact(&index, entry, &libraries, self.client.as_ref(), base).await?;
            }
        }
        reporter.check_cancelled()?;

        // Data entries name generated files as well as real downloads: resolve them to
        // local paths, never fetch them.
        let mut data: BTreeMap<String, String> = BTreeMap::new();
        for (key, sides) in &profile.data {
            if sides.client.is_empty() {
                continue;
            }
            data.insert(
                key.clone(),
                resolve_data_value(&sides.client, &installer, &work, &libraries, &index).await?,
            );
        }
        let mut placeholders = data;
        placeholders.insert(
            String::from("MINECRAFT_JAR"),
            context.vanilla_jar.to_string_lossy().into_owned(),
        );
        placeholders.insert(
            String::from("MINECRAFT_VERSION"),
            context.game_version.clone(),
        );
        placeholders.insert(String::from("SIDE"), String::from("client"));
        placeholders.insert(
            String::from("ROOT"),
            context.game_paths.root().to_string_lossy().into_owned(),
        );
        placeholders.insert(
            String::from("INSTALLER"),
            installer.to_string_lossy().into_owned(),
        );
        placeholders.insert(
            String::from("LIBRARY_DIR"),
            libraries.to_string_lossy().into_owned(),
        );

        let client_processors: Vec<&ProcessorSpec> = profile
            .processors
            .iter()
            .filter(|processor| processor.runs_on_client())
            .collect();
        for (position, processor) in client_processors.iter().enumerate() {
            reporter.check_cancelled()?;
            reporter.progress(
                25 + (position as u64 * 65) / client_processors.len().max(1) as u64,
                Some(100),
                &format!("Running installer processor {}", position + 1),
            );
            run_processor(processor, context, &libraries, &index, &placeholders).await?;
            verify_outputs(processor, &placeholders).await?;
        }

        let _ = tokio::fs::remove_dir_all(&work).await;
        reporter.progress(95, Some(100), "Finishing the loader install");
        Ok(document)
    }
}

async fn run_processor(
    processor: &ProcessorSpec,
    context: &LoaderInstallContext,
    libraries: &Path,
    index: &BTreeMap<String, DownloadInfo>,
    placeholders: &BTreeMap<String, String>,
) -> CoreResult<()> {
    let jar_path = library_path(index, &processor.jar, libraries);
    let main_class = read_main_class(&jar_path).await?;

    let mut classpath = vec![jar_path.to_string_lossy().into_owned()];
    for entry in &processor.classpath {
        classpath.push(
            library_path(index, entry, libraries)
                .to_string_lossy()
                .into_owned(),
        );
    }
    let args: Vec<String> = processor
        .args
        .iter()
        .map(|argument| expand(argument, placeholders, libraries))
        .collect();

    let output = tokio::time::timeout(
        PROCESSOR_TIMEOUT,
        Command::new(&context.java_path)
            .arg("-cp")
            .arg(classpath.join(classpath_separator()))
            .arg(main_class)
            .args(args)
            .current_dir(context.game_paths.root())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .map_err(|_| {
        CoreError::unavailable(format!(
            "the installer processor {} timed out",
            processor.jar
        ))
    })?
    .map_err(|error| {
        CoreError::io(format!(
            "could not run the installer processor {}",
            processor.jar
        ))
        .with_details(error.to_string())
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let tail: String = format!("{stdout}\n{stderr}")
            .lines()
            .rev()
            .take(40)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(CoreError::internal(format!(
            "the installer processor {} failed",
            processor.jar
        ))
        .with_details(tail));
    }
    Ok(())
}

async fn verify_outputs(
    processor: &ProcessorSpec,
    placeholders: &BTreeMap<String, String>,
) -> CoreResult<()> {
    for (name, expected) in &processor.outputs {
        let path = expand(name, placeholders, Path::new(""));
        let expected = expand(expected, placeholders, Path::new(""));
        if !looks_like_sha1(&expected) {
            continue;
        }
        let bytes = tokio::fs::read(&path).await.map_err(|error| {
            CoreError::io(format!("the installer did not produce {path}"))
                .with_details(error.to_string())
        })?;
        let digest = Sha1::digest(&bytes);
        let actual = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if !actual.eq_ignore_ascii_case(&expected) {
            return Err(CoreError::internal(format!(
                "{path} did not match the hash the installer published"
            )));
        }
    }
    Ok(())
}

fn looks_like_sha1(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn expand(template: &str, placeholders: &BTreeMap<String, String>, libraries: &Path) -> String {
    let mut expanded = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        expanded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            expanded.push_str(&rest[start..]);
            return expanded;
        };
        let key = &after[..end];
        match placeholders.get(key) {
            Some(value) => expanded.push_str(value),
            None => {
                expanded.push('{');
                expanded.push_str(key);
                expanded.push('}');
            }
        }
        rest = &after[end + 1..];
    }
    expanded.push_str(rest);
    expand_maven_tokens(&expanded, libraries)
}

/// Replaces `[group:artifact:version]` tokens with their path below `libraries`.
fn expand_maven_tokens(value: &str, libraries: &Path) -> String {
    let mut expanded = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('[') {
        expanded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find(']') else {
            expanded.push_str(&rest[start..]);
            return expanded;
        };
        let token = &after[..end];
        match MavenCoordinate::parse(token) {
            Ok(coordinate) if !libraries.as_os_str().is_empty() => {
                expanded.push_str(
                    &libraries
                        .join(coordinate.repository_path())
                        .to_string_lossy(),
                );
            }
            _ => {
                expanded.push('[');
                expanded.push_str(token);
                expanded.push(']');
            }
        }
        rest = &after[end + 1..];
    }
    expanded.push_str(rest);
    expanded
}

fn library_index(libraries: &[Library]) -> BTreeMap<String, DownloadInfo> {
    let mut index = BTreeMap::new();
    for library in libraries {
        if let Some(artifact) = library
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.artifact.as_ref())
        {
            index.insert(library.name.clone(), artifact.clone());
        }
    }
    index
}

fn library_path(
    index: &BTreeMap<String, DownloadInfo>,
    coordinate: &str,
    libraries: &Path,
) -> PathBuf {
    let relative = index
        .get(coordinate)
        .and_then(|download| download.path.clone())
        .or_else(|| {
            MavenCoordinate::parse(coordinate)
                .ok()
                .map(|parsed| parsed.repository_path())
        })
        .unwrap_or_else(|| coordinate.to_owned());
    libraries.join(relative)
}

async fn ensure_artifact(
    index: &BTreeMap<String, DownloadInfo>,
    coordinate: &str,
    libraries: &Path,
    client: &dyn HttpClient,
    base: &str,
) -> CoreResult<PathBuf> {
    let download = index.get(coordinate);
    let destination = library_path(index, coordinate, libraries);
    if destination.is_file() {
        return Ok(destination);
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(|error| {
            CoreError::io("could not create the libraries directory")
                .with_details(error.to_string())
        })?;
    }
    let url = match download {
        Some(download) => download.url.clone(),
        None => {
            let parsed = MavenCoordinate::parse(coordinate)?;
            format!("{base}{}", parsed.repository_path())
        }
    };
    let bytes = fetch_bytes(client, &url).await?;
    if let Some(expected) = download.and_then(|download| download.sha1.as_deref()) {
        let digest = Sha1::digest(&bytes);
        let actual = digest
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(CoreError::network(format!(
                "{coordinate} did not match its published hash"
            )));
        }
    }
    tokio::fs::write(&destination, &bytes)
        .await
        .map_err(|error| {
            CoreError::io(format!("could not save {coordinate}")).with_details(error.to_string())
        })?;
    Ok(destination)
}

async fn resolve_data_value(
    value: &str,
    installer: &Path,
    work: &Path,
    libraries: &Path,
    index: &BTreeMap<String, DownloadInfo>,
) -> CoreResult<String> {
    let value = value.trim();
    if let Some(inner) = value
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    {
        return Ok(inner.to_owned());
    }
    if let Some(inner) = value
        .strip_prefix('[')
        .and_then(|rest| rest.strip_suffix(']'))
    {
        // Generated artefacts (unpacked, slim, patched, …) only get a path; every real
        // download in the profile was fetched before the data map was resolved.
        return Ok(library_path(index, inner, libraries)
            .to_string_lossy()
            .into_owned());
    }
    if let Some(entry) = value.strip_prefix('/') {
        let archive = installer.to_path_buf();
        let destination = work.join(
            Path::new(entry)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("data.bin")),
        );
        let entry = entry.to_owned();
        let destination_for_task = destination.clone();
        tokio::task::spawn_blocking(move || {
            extract_zip_entry(&archive, &entry, &destination_for_task)
        })
        .await
        .map_err(|error| {
            CoreError::internal("the installer data extractor failed")
                .with_details(error.to_string())
        })??;
        return Ok(destination.to_string_lossy().into_owned());
    }
    Ok(value.to_owned())
}

fn read_installer_documents(archive: &Path) -> CoreResult<(Vec<u8>, Vec<u8>)> {
    let file = std::fs::File::open(archive).map_err(|error| {
        CoreError::io(format!("could not open {}", archive.display()))
            .with_details(error.to_string())
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|error| {
        CoreError::network("the loader installer is damaged").with_details(error.to_string())
    })?;
    let profile = read_entry(&mut zip, "install_profile.json")?;
    let path = serde_json::from_slice::<serde_json::Value>(&profile)
        .ok()
        .and_then(|value| {
            value
                .get("json")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| String::from("/version.json"));
    let profile_path = path.trim_start_matches('/');
    let version = read_entry(&mut zip, profile_path)?;
    Ok((profile, version))
}

fn read_entry(zip: &mut zip::ZipArchive<std::fs::File>, name: &str) -> CoreResult<Vec<u8>> {
    let mut entry = zip.by_name(name).map_err(|error| {
        CoreError::network(format!("the installer has no {name}")).with_details(error.to_string())
    })?;
    let mut bytes = Vec::new();
    entry.read_to_end(&mut bytes).map_err(|error| {
        CoreError::io("could not read from the installer").with_details(error.to_string())
    })?;
    Ok(bytes)
}

fn extract_maven(archive: &Path, libraries: &Path) -> CoreResult<usize> {
    let file = std::fs::File::open(archive).map_err(|error| {
        CoreError::io(format!("could not open {}", archive.display()))
            .with_details(error.to_string())
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|error| {
        CoreError::network("the loader installer is damaged").with_details(error.to_string())
    })?;
    let mut written = 0usize;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|error| {
            CoreError::io("an installer entry is damaged").with_details(error.to_string())
        })?;
        if entry.is_dir() {
            continue;
        }
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        let Ok(relative) = relative.strip_prefix("maven") else {
            continue;
        };
        let target = libraries.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                CoreError::io("could not create a library directory")
                    .with_details(error.to_string())
            })?;
        }
        let mut output = std::fs::File::create(&target).map_err(|error| {
            CoreError::io(format!("could not write {}", target.display()))
                .with_details(error.to_string())
        })?;
        std::io::copy(&mut entry, &mut output).map_err(|error| {
            CoreError::io("could not unpack a library").with_details(error.to_string())
        })?;
        written += 1;
    }
    Ok(written)
}

fn extract_zip_entry(archive: &Path, name: &str, destination: &Path) -> CoreResult<()> {
    let file = std::fs::File::open(archive).map_err(|error| {
        CoreError::io(format!("could not open {}", archive.display()))
            .with_details(error.to_string())
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(|error| {
        CoreError::network("the loader installer is damaged").with_details(error.to_string())
    })?;
    let mut entry = zip.by_name(name).map_err(|error| {
        CoreError::network(format!("the installer has no {name}")).with_details(error.to_string())
    })?;
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            CoreError::io("could not create the work directory").with_details(error.to_string())
        })?;
    }
    let mut output = std::fs::File::create(destination).map_err(|error| {
        CoreError::io(format!("could not write {}", destination.display()))
            .with_details(error.to_string())
    })?;
    std::io::copy(&mut entry, &mut output).map_err(|error| {
        CoreError::io("could not unpack installer data").with_details(error.to_string())
    })?;
    Ok(())
}

async fn read_main_class(jar: &Path) -> CoreResult<String> {
    let jar = jar.to_path_buf();
    tokio::task::spawn_blocking(move || -> CoreResult<String> {
        let file = std::fs::File::open(&jar).map_err(|error| {
            CoreError::io(format!("could not open {}", jar.display()))
                .with_details(error.to_string())
        })?;
        let mut zip = zip::ZipArchive::new(file).map_err(|error| {
            CoreError::network("a processor jar is damaged").with_details(error.to_string())
        })?;
        let mut entry = zip.by_name("META-INF/MANIFEST.MF").map_err(|error| {
            CoreError::network(format!("{} has no manifest", jar.display()))
                .with_details(error.to_string())
        })?;
        let mut manifest = String::new();
        entry.read_to_string(&mut manifest).map_err(|error| {
            CoreError::io("could not read a processor manifest").with_details(error.to_string())
        })?;
        let mut current = String::new();
        let mut main_class = None;
        for line in manifest.lines() {
            if let Some(continuation) = line.strip_prefix(' ') {
                current.push_str(continuation);
                continue;
            }
            if let Some(value) = current.strip_prefix("Main-Class:") {
                main_class = Some(value.trim().to_owned());
            }
            current.clear();
            current.push_str(line);
        }
        if let Some(value) = current.strip_prefix("Main-Class:") {
            main_class = Some(value.trim().to_owned());
        }
        main_class
            .filter(|value| !value.is_empty())
            .ok_or_else(|| CoreError::network(format!("{} names no main class", jar.display())))
    })
    .await
    .map_err(|error| {
        CoreError::internal("the processor manifest reader failed").with_details(error.to_string())
    })?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neo_forge_prefixes_follow_the_game_version() {
        assert_eq!(neoforge_prefix("1.21.4").as_deref(), Some("21.4."));
        assert_eq!(neoforge_prefix("1.21.1").as_deref(), Some("21.1."));
        assert_eq!(neoforge_prefix("1.20.6").as_deref(), Some("20.6."));
        assert_eq!(neoforge_prefix("1.21").as_deref(), Some("21.0."));
        assert_eq!(neoforge_prefix("26.3").as_deref(), Some("26.3."));
        assert_eq!(neoforge_prefix("26.1.2").as_deref(), Some("26.1.2."));
    }

    #[test]
    fn forge_artifact_versions_do_not_double_the_game_prefix() {
        assert_eq!(
            artifact_version(LoaderKind::Forge, "1.20.1", "47.2.0"),
            "1.20.1-47.2.0"
        );
        assert_eq!(
            artifact_version(LoaderKind::Forge, "1.20.1", "1.20.1-47.2.0"),
            "1.20.1-47.2.0"
        );
    }

    #[test]
    fn the_installer_url_points_at_the_expected_artifact() {
        let url = installer_url(LoaderKind::Forge, "1.20.1", "47.2.0").expect("url");
        assert!(url.ends_with("forge-1.20.1-47.2.0-installer.jar"));
        let url = installer_url(LoaderKind::NeoForge, "1.21.4", "21.4.161").expect("url");
        assert!(url.ends_with("neoforge-21.4.161-installer.jar"));
    }

    #[test]
    fn maven_metadata_versions_are_extracted() {
        let xml = "<metadata><versioning><versions>\
                   <version>1.20.1-47.2.0</version>\
                   <version>1.20.1-47.3.0</version>\
                   </versions></versioning></metadata>";
        assert_eq!(
            parse_metadata_versions(xml),
            vec!["1.20.1-47.2.0", "1.20.1-47.3.0"]
        );
    }

    #[test]
    fn versions_sort_numerically() {
        let mut versions = vec!["21.4.9", "21.4.161", "21.4.10"];
        versions.sort_by_key(|version| std::cmp::Reverse(version_key(version)));
        assert_eq!(versions, vec!["21.4.161", "21.4.10", "21.4.9"]);
    }

    #[test]
    fn placeholders_and_maven_tokens_expand() {
        let placeholders = BTreeMap::from([
            (String::from("SIDE"), String::from("client")),
            (
                String::from("MC_SLIM"),
                String::from("/libs/client-slim.jar"),
            ),
        ]);
        assert_eq!(
            expand(
                "--side {SIDE} --slim {MC_SLIM}",
                &placeholders,
                Path::new("")
            ),
            "--side client --slim /libs/client-slim.jar"
        );
        let expected = format!(
            "--input {}",
            Path::new("/root/libraries")
                .join("net/minecraft/client/1.20.1/client-1.20.1-slim.jar")
                .display()
        );
        assert_eq!(
            expand(
                "--input [net.minecraft:client:1.20.1:slim@jar]",
                &placeholders,
                Path::new("/root/libraries")
            ),
            expected
        );
    }

    #[test]
    fn an_unknown_placeholder_stays_visible() {
        let placeholders: BTreeMap<String, String> = BTreeMap::new();
        assert_eq!(
            expand("{MYSTERY} stays", &placeholders, Path::new("")),
            "{MYSTERY} stays"
        );
    }

    #[test]
    fn only_client_or_unrestricted_processors_run() {
        let client = ProcessorSpec {
            sides: Vec::new(),
            jar: String::new(),
            classpath: Vec::new(),
            args: Vec::new(),
            outputs: BTreeMap::new(),
        };
        assert!(client.runs_on_client());
        let server_only = ProcessorSpec {
            sides: vec![String::from("server")],
            ..client
        };
        assert!(!server_only.runs_on_client());
    }

    #[test]
    fn sha1_looking_values_are_recognised() {
        assert!(looks_like_sha1("de86b035d2da0f78940796bb95c39a932ed84834"));
        assert!(!looks_like_sha1("1.20.1"));
    }
}
