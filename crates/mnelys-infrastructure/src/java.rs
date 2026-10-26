//! Java discovery and managed runtime installation.
//!
//! Discovery probes `JAVA_HOME`, the PATH, the runtimes Mnelys installed and the usual
//! install directories. Every candidate is executed with `-XshowSettings:properties`
//! so the reported major and vendor come from the JVM itself, not from a folder name.
//!
//! Managed runtimes are Eclipse Temurin builds fetched from the Adoptium API, checked
//! against the published SHA-256 and unpacked under `<root>/java/<major>`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use mnelys_application::paths::GamePaths;
use mnelys_application::ports::{fetch_bytes, fetch_json, HttpClient, JavaProvider};
use mnelys_application::TaskReporter;
use mnelys_domain::{CoreError, CoreResult, JavaRuntime, JavaSource};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::process::Command;

const ADOPTIUM_LATEST: &str = "https://api.adoptium.net/v3/assets/latest";
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

/// Discovers local runtimes and installs Temurin builds.
pub struct LocalJavaProvider {
    client: Arc<dyn HttpClient>,
    paths: GamePaths,
}

impl LocalJavaProvider {
    #[must_use]
    pub fn new(client: Arc<dyn HttpClient>, paths: GamePaths) -> Self {
        Self { client, paths }
    }

    async fn candidates(&self) -> Vec<(PathBuf, JavaSource)> {
        let mut candidates = Vec::new();
        let mut seen = BTreeSet::new();

        let mut push = |path: PathBuf, source: JavaSource| {
            if seen.insert(path.clone()) {
                candidates.push((path, source));
            }
        };

        if let Some(home) = std::env::var_os("JAVA_HOME") {
            push(executable_in(Path::new(&home)), JavaSource::System);
        }

        // Managed runtimes: `<root>/java/<major>/bin/java`.
        if let Ok(mut entries) = tokio::fs::read_dir(self.paths.java_dir()).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if entry
                    .file_type()
                    .await
                    .map(|kind| kind.is_dir())
                    .unwrap_or(false)
                {
                    push(executable_in(&path), JavaSource::Managed);
                }
            }
        }

        for root in known_roots() {
            let mut homes = Vec::new();
            collect_homes(&root, 0, &mut homes).await;
            for home in homes {
                push(executable_in(&home), JavaSource::System);
            }
        }

        if let Some(path) = std::env::var_os("PATH") {
            for directory in std::env::split_paths(&path) {
                let executable = directory.join(java_binary_name());
                if tokio::fs::metadata(&executable).await.is_ok() {
                    push(executable, JavaSource::System);
                }
            }
        }

        candidates
    }

    /// Finds one managed runtime for a major version, installing it when absent.
    async fn existing_managed(&self, major: u32) -> Option<JavaRuntime> {
        self.discover()
            .await
            .ok()?
            .into_iter()
            .find(|runtime| runtime.major == major && runtime.source == JavaSource::Managed)
    }
}

#[async_trait]
impl JavaProvider for LocalJavaProvider {
    async fn discover(&self) -> CoreResult<Vec<JavaRuntime>> {
        let mut runtimes = Vec::new();
        let mut seen = BTreeSet::new();
        for (path, source) in self.candidates().await {
            let key = tokio::fs::canonicalize(&path)
                .await
                .unwrap_or_else(|_| path.clone());
            if !seen.insert(key) {
                continue;
            }
            if let Some((version, major, vendor)) = probe(&path).await {
                runtimes.push(JavaRuntime {
                    path: path.to_string_lossy().into_owned(),
                    major,
                    version,
                    vendor,
                    source,
                });
            }
        }
        runtimes.sort_by(|left, right| {
            left.major
                .cmp(&right.major)
                .then_with(|| left.rank().cmp(&right.rank()))
                .then_with(|| left.path.cmp(&right.path))
        });
        Ok(runtimes)
    }

    async fn install(&self, major: u32, reporter: &dyn TaskReporter) -> CoreResult<JavaRuntime> {
        if !(8..=64).contains(&major) {
            return Err(CoreError::invalid_request(
                "Java major version must be between 8 and 64",
            ));
        }
        if let Some(runtime) = self.existing_managed(major).await {
            return Ok(runtime);
        }

        reporter.progress(0, None, "Resolving a Java runtime");
        let url = adoptium_url(major);
        let assets: Vec<AdoptiumAsset> = fetch_json(self.client.as_ref(), &url).await?;
        let package = assets
            .into_iter()
            .next()
            .ok_or_else(|| {
                CoreError::unavailable(format!("no Temurin {major} build exists for this platform"))
            })?
            .binary
            .package;

        reporter.progress(0, None, &format!("Downloading {}", package.name));
        let bytes = fetch_bytes(self.client.as_ref(), &package.link).await?;
        reporter.check_cancelled()?;
        verify_sha256(&bytes, &package.checksum)?;

        let downloads = self.paths.java_dir().join("downloads");
        tokio::fs::create_dir_all(&downloads)
            .await
            .map_err(|error| {
                CoreError::io("could not create the Java download directory")
                    .with_details(error.to_string())
            })?;
        let archive = downloads.join(&package.name);
        tokio::fs::write(&archive, &bytes).await.map_err(|error| {
            CoreError::io("could not write the Java archive").with_details(error.to_string())
        })?;

        reporter.progress(50, Some(100), "Unpacking the runtime");
        let staging = self.paths.java_dir().join(format!(
            ".install-{major}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        ));
        let archive_for_task = archive.clone();
        let staging_for_task = staging.clone();
        tokio::task::spawn_blocking(move || unpack(&archive_for_task, &staging_for_task))
            .await
            .map_err(|error| {
                CoreError::internal("the Java archive task failed").with_details(error.to_string())
            })??;
        reporter.check_cancelled()?;

        let home = find_java_home(&staging).ok_or_else(|| {
            CoreError::io("the downloaded Java archive does not contain a runtime")
        })?;
        let target = self.paths.java_runtime_dir(major);
        let _ = tokio::fs::remove_dir_all(&target).await;
        tokio::fs::create_dir_all(self.paths.java_dir())
            .await
            .map_err(|error| {
                CoreError::io("could not create the Java directory").with_details(error.to_string())
            })?;
        tokio::fs::rename(&home, &target).await.map_err(|error| {
            CoreError::io("the Java runtime could not be moved into place")
                .with_details(error.to_string())
        })?;
        let _ = tokio::fs::remove_dir_all(&staging).await;
        let _ = tokio::fs::remove_file(&archive).await;

        reporter.progress(90, Some(100), "Checking the runtime");
        let executable = executable_in(&target);
        let (version, found_major, vendor) = probe(&executable)
            .await
            .ok_or_else(|| CoreError::io("the installed Java runtime does not start"))?;
        reporter.progress(100, Some(100), "Java runtime ready");
        Ok(JavaRuntime {
            path: executable.to_string_lossy().into_owned(),
            major: found_major,
            version,
            vendor,
            source: JavaSource::Managed,
        })
    }
}

#[derive(Debug, Deserialize)]
struct AdoptiumAsset {
    binary: AdoptiumBinary,
}

#[derive(Debug, Deserialize)]
struct AdoptiumBinary {
    package: AdoptiumPackage,
}

#[derive(Debug, Deserialize)]
struct AdoptiumPackage {
    name: String,
    link: String,
    checksum: String,
}

fn java_binary_name() -> &'static str {
    if cfg!(windows) {
        "java.exe"
    } else {
        "java"
    }
}

fn executable_in(home: &Path) -> PathBuf {
    home.join("bin").join(java_binary_name())
}

async fn collect_homes(root: &Path, depth: usize, homes: &mut Vec<PathBuf>) {
    if depth > 2 {
        return;
    }
    let Ok(mut entries) = tokio::fs::read_dir(root).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if !entry
            .file_type()
            .await
            .map(|kind| kind.is_dir())
            .unwrap_or(false)
        {
            continue;
        }
        if tokio::fs::metadata(executable_in(&path)).await.is_ok() {
            homes.push(path);
            continue;
        }
        // macOS bundles put the home one level below the `.jdk` directory.
        let contents = path.join("Contents").join("Home");
        if tokio::fs::metadata(executable_in(&contents)).await.is_ok() {
            homes.push(contents);
            continue;
        }
        Box::pin(collect_homes(&path, depth + 1, homes)).await;
    }
}

#[cfg(windows)]
fn known_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let program_files = PathBuf::from(program_files);
        for vendor in [
            "Java",
            "Eclipse Adoptium",
            "Eclipse Foundation",
            "Microsoft",
            "BellSoft",
            "Zulu",
            "Amazon Corretto",
            "Semeru",
            "AdoptOpenJDK",
        ] {
            roots.push(program_files.join(vendor));
        }
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        roots.push(
            PathBuf::from(local)
                .join("Programs")
                .join("Eclipse Adoptium"),
        );
    }
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".jdks"));
    }
    roots
}

#[cfg(target_os = "macos")]
fn known_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Library/Java/JavaVirtualMachines"),
        PathBuf::from("/System/Library/Java/JavaVirtualMachines"),
    ];
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join("Library/Java/JavaVirtualMachines"));
        roots.push(home.join(".jdks"));
        roots.push(home.join(".sdkman/candidates/java"));
    }
    roots
}

#[cfg(all(unix, not(target_os = "macos")))]
fn known_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/usr/lib/jvm"),
        PathBuf::from("/usr/java"),
        PathBuf::from("/opt/java"),
    ];
    if let Some(home) = dirs::home_dir() {
        roots.push(home.join(".jdks"));
        roots.push(home.join(".sdkman/candidates/java"));
    }
    roots
}

async fn probe(executable: &Path) -> Option<(String, u32, String)> {
    let output = tokio::time::timeout(
        PROBE_TIMEOUT,
        Command::new(executable)
            .arg("-XshowSettings:properties")
            .arg("-version")
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .ok()?
    .ok()?;
    parse_properties(&String::from_utf8_lossy(&output.stderr))
}

fn parse_properties(text: &str) -> Option<(String, u32, String)> {
    let mut version = None;
    let mut specification = None;
    let mut vendor = None;
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("java.version = ") {
            version = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("java.specification.version = ") {
            specification = Some(value.trim().to_owned());
        } else if let Some(value) = line.strip_prefix("java.vendor = ") {
            vendor = Some(value.trim().to_owned());
        }
    }
    let version = version?;
    let major = specification
        .as_deref()
        .and_then(major_from)
        .or_else(|| major_from(&version))?;
    Some((
        version,
        major,
        vendor.unwrap_or_else(|| String::from("Unknown")),
    ))
}

fn major_from(version: &str) -> Option<u32> {
    let mut parts = version.split('.');
    let first = parts.next()?;
    let major = if first == "1" { parts.next()? } else { first };
    major.parse().ok()
}

fn adoptium_url(major: u32) -> String {
    let os = if cfg!(windows) {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "aarch64",
        "x86" => "x86",
        other => other,
    };
    format!(
        "{ADOPTIUM_LATEST}/{major}/hotspot?architecture={arch}&image_type=jdk&os={os}&vendor=eclipse"
    )
}

fn verify_sha256(bytes: &[u8], expected: &str) -> CoreResult<()> {
    let digest = Sha256::digest(bytes);
    let actual = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(CoreError::network(
            "the downloaded Java archive did not match its published checksum",
        ))
    }
}

fn unpack(archive: &Path, destination: &Path) -> CoreResult<()> {
    std::fs::create_dir_all(destination).map_err(|error| {
        CoreError::io("could not create the Java staging directory").with_details(error.to_string())
    })?;
    let file = std::fs::File::open(archive).map_err(|error| {
        CoreError::io(format!("could not open {}", archive.display()))
            .with_details(error.to_string())
    })?;
    if archive
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        let mut zip = zip::ZipArchive::new(file).map_err(|error| {
            CoreError::io("the Java archive is damaged").with_details(error.to_string())
        })?;
        zip.extract(destination).map_err(|error| {
            CoreError::io("the Java archive could not be unpacked").with_details(error.to_string())
        })
    } else {
        let decoder = flate2::read::GzDecoder::new(file);
        let mut tar = tar::Archive::new(decoder);
        tar.unpack(destination).map_err(|error| {
            CoreError::io("the Java archive could not be unpacked").with_details(error.to_string())
        })
    }
}

fn find_java_home(root: &Path) -> Option<PathBuf> {
    let mut queue = vec![(root.to_path_buf(), 0usize)];
    while let Some((directory, depth)) = queue.pop() {
        if depth > 4 {
            continue;
        }
        if directory.join("bin").join(java_binary_name()).is_file() {
            return Some(directory);
        }
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                queue.push((entry.path(), depth + 1));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn java_8_versions_map_to_major_8() {
        assert_eq!(major_from("1.8.0_402"), Some(8));
        assert_eq!(major_from("8.0.412"), Some(8));
    }

    #[test]
    fn modern_versions_map_to_their_first_component() {
        assert_eq!(major_from("21.0.4+7-LTS"), Some(21));
        assert_eq!(major_from("17"), Some(17));
    }

    #[test]
    fn properties_are_parsed_from_jvm_output() {
        let text = "Property settings:\n    java.version = 21.0.4\n    \
                    java.specification.version = 21\n    java.vendor = Eclipse Adoptium\n";
        let (version, major, vendor) = parse_properties(text).expect("parses");
        assert_eq!(version, "21.0.4");
        assert_eq!(major, 21);
        assert_eq!(vendor, "Eclipse Adoptium");
    }

    #[test]
    fn output_without_a_version_is_rejected() {
        assert!(parse_properties("no java here").is_none());
    }

    #[test]
    fn a_matching_checksum_is_accepted() {
        // SHA-256 of the empty input.
        let checksum = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert!(verify_sha256(b"", checksum).is_ok());
        assert!(verify_sha256(b"x", checksum).is_err());
    }

    #[test]
    fn the_adoptium_url_names_the_major_and_platform() {
        let url = adoptium_url(21);
        assert!(url.contains("/21/hotspot"));
        assert!(url.contains("image_type=jdk"));
    }

    #[test]
    fn the_executable_name_matches_the_platform() {
        let expected = if cfg!(windows) { "java.exe" } else { "java" };
        assert_eq!(java_binary_name(), expected);
    }
}
