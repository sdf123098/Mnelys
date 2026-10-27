//! Browsing the files one instance owns: contents, worlds, schematics and logs.
//!
//! Every path the interface names lives below `<instance>/game`. The relative path is
//! parsed before it touches the disk — absolute paths, drive letters and `..` are
//! rejected — and the result is then compared with the canonical instance root, so a
//! symlink planted inside the instance cannot aim the file manager at the rest of the
//! machine.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use mnelys_domain::{validate_slug, CoreError, CoreResult};
use serde::Serialize;

use crate::instances::InstanceService;
use crate::paths::GamePaths;
use crate::ports::{DirEntry, EntryRevealer, FileStore};

/// Log endings the log panel lists and reads.
const LOG_EXTENSIONS: [&str; 2] = ["log", "txt"];
/// Schematic formats the projector panel recognises.
const SCHEMATIC_EXTENSIONS: [&str; 3] = ["litem", "schem", "schematic"];
/// How much of a log file the reader takes from its end.
const LOG_TAIL_BYTES: u64 = 256 * 1024;
/// Entries one listing returns before the rest are marked as not shown.
const MAX_LISTED_ENTRIES: usize = 2_000;
/// How deep a recursive walk goes when it measures a world or looks for schematics.
const MAX_WALK_DEPTH: usize = 6;
/// Files one recursive walk counts before it reports a lower bound instead.
const MAX_WALK_FILES: usize = 20_000;
/// Longest relative path the service accepts.
const MAX_RELATIVE_LENGTH: usize = 1_024;

/// One file or directory inside an instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEntry {
    pub name: String,
    /// Slash separated path below the instance game directory.
    pub path: String,
    pub is_dir: bool,
    /// Always zero for a directory: measuring one is a separate, expensive walk.
    pub size_bytes: u64,
    /// `None` when the platform could not read the timestamp.
    pub modified_ms: Option<u64>,
}

/// One directory listing inside an instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileListing {
    /// The listed directory; empty for the instance root.
    pub path: String,
    pub entries: Vec<WorkspaceEntry>,
    /// True when the directory holds more entries than a listing returns.
    pub truncated: bool,
}

/// What the log panel shows: launcher logs and crash reports, kept apart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogListing {
    pub logs: Vec<WorkspaceEntry>,
    pub crash_reports: Vec<WorkspaceEntry>,
}

/// The end of one log file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogTail {
    pub path: String,
    pub text: String,
    /// True when the file is longer than the window the core read.
    pub truncated: bool,
}

/// One saved world below `saves/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    /// False when the walk stopped early, which makes `size_bytes` a lower bound.
    pub size_complete: bool,
    pub modified_ms: Option<u64>,
}

/// A path that has been proved to live inside one instance.
struct Located {
    /// The canonical instance game directory, used to write relative display paths.
    root: PathBuf,
    /// The resolved target, safe to hand to the filesystem.
    path: PathBuf,
    /// The same target below the friendly root, for the desktop file manager.
    desktop_path: PathBuf,
}

/// Reads and prunes the files of one instance directory.
pub struct WorkspaceService {
    files: Arc<dyn FileStore>,
    paths: GamePaths,
    instances: Arc<InstanceService>,
    revealer: Arc<dyn EntryRevealer>,
}

impl WorkspaceService {
    #[must_use]
    pub fn new(
        files: Arc<dyn FileStore>,
        paths: GamePaths,
        instances: Arc<InstanceService>,
        revealer: Arc<dyn EntryRevealer>,
    ) -> Self {
        Self {
            files,
            paths,
            instances,
            revealer,
        }
    }

    /// Lists one directory below the instance root. An empty `dir` lists the root.
    pub async fn list(&self, instance_id: &str, dir: &str) -> CoreResult<FileListing> {
        let located = self.locate(instance_id, dir).await?;
        let entries = self.files.list(&located.path).await?;
        let mut listed: Vec<WorkspaceEntry> = entries
            .iter()
            .map(|entry| workspace_entry(&located.root, entry))
            .collect();
        listed.sort_by(|left, right| {
            right
                .is_dir
                .cmp(&left.is_dir)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
        });
        let truncated = listed.len() > MAX_LISTED_ENTRIES;
        listed.truncate(MAX_LISTED_ENTRIES);
        Ok(FileListing {
            path: relative_of(&located.root, &located.path),
            entries: listed,
            truncated,
        })
    }

    /// Lists the launcher logs and the crash reports of one instance.
    ///
    /// A freshly created instance has neither, so an absent directory is an empty list
    /// rather than a failure.
    pub async fn logs(&self, instance_id: &str) -> CoreResult<LogListing> {
        Ok(LogListing {
            logs: self
                .list_known(instance_id, "logs", &LOG_EXTENSIONS)
                .await?,
            crash_reports: self
                .list_known(instance_id, "crash-reports", &LOG_EXTENSIONS)
                .await?,
        })
    }

    /// Reads the end of one log file.
    pub async fn read_log(&self, instance_id: &str, relative: &str) -> CoreResult<LogTail> {
        let located = self.locate(instance_id, relative).await?;
        if located.path == located.root || !has_extension(&located.path, &LOG_EXTENSIONS) {
            return Err(CoreError::invalid_request(
                "only log and text files can be read here",
            ));
        }
        let size = self.files.len(&located.path).await.unwrap_or(0);
        let bytes = self.files.read_tail(&located.path, LOG_TAIL_BYTES).await?;
        let read = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        Ok(LogTail {
            path: relative_of(&located.root, &located.path),
            text: tail_text(&bytes, size > read),
            truncated: size > read,
        })
    }

    /// Lists the saved worlds, with the size the core could actually measure.
    pub async fn worlds(&self, instance_id: &str) -> CoreResult<Vec<WorldEntry>> {
        let root = self.roots(instance_id).await?.0;
        let entries = self.files.list(&root.join("saves")).await?;
        let mut worlds = Vec::new();
        for entry in entries.iter().filter(|entry| entry.is_dir) {
            let (size_bytes, size_complete) = self.measure(&entry.path).await;
            worlds.push(WorldEntry {
                name: entry.name.clone(),
                path: relative_of(&root, &entry.path),
                size_bytes,
                size_complete,
                modified_ms: entry.modified_ms,
            });
        }
        worlds.sort_by(|left, right| {
            right
                .modified_ms
                .unwrap_or(0)
                .cmp(&left.modified_ms.unwrap_or(0))
                .then_with(|| left.name.cmp(&right.name))
        });
        Ok(worlds)
    }

    /// Finds schematic files below `schematics/`, including the subfolders the mods
    /// create there.
    pub async fn schematics(&self, instance_id: &str) -> CoreResult<Vec<WorkspaceEntry>> {
        let root = self.roots(instance_id).await?.0;
        let mut found = self
            .collect_files(
                root.join("schematics"),
                &root,
                SCHEMATIC_EXTENSIONS.as_slice(),
            )
            .await?;
        found.sort_by(|left, right| left.path.cmp(&right.path));
        Ok(found)
    }

    /// Deletes one file, or one whole world folder, inside the instance.
    pub async fn delete(&self, instance_id: &str, relative: &str) -> CoreResult<()> {
        let located = self.locate(instance_id, relative).await?;
        if located.path == located.root {
            return Err(CoreError::invalid_request(
                "the instance directory itself is not deleted here",
            ));
        }
        if self.files.is_directory(&located.path).await {
            self.files.remove_dir_all(&located.path).await
        } else {
            self.files.remove(&located.path).await
        }
    }

    /// Shows one entry in the desktop file manager.
    pub async fn reveal(&self, instance_id: &str, relative: &str) -> CoreResult<()> {
        let located = self.locate(instance_id, relative).await?;
        self.revealer.reveal(&located.desktop_path)
    }

    /// Lists one well-known subdirectory, which may not exist yet.
    async fn list_known(
        &self,
        instance_id: &str,
        directory: &str,
        extensions: &[&str],
    ) -> CoreResult<Vec<WorkspaceEntry>> {
        let root = self.roots(instance_id).await?.0;
        let entries = self.files.list(&root.join(directory)).await?;
        let mut listed: Vec<WorkspaceEntry> = entries
            .iter()
            .filter(|entry| !entry.is_dir && has_extension(&entry.path, extensions))
            .map(|entry| workspace_entry(&root, entry))
            .collect();
        listed.sort_by(|left, right| {
            right
                .modified_ms
                .unwrap_or(0)
                .cmp(&left.modified_ms.unwrap_or(0))
                .then_with(|| left.name.cmp(&right.name))
        });
        listed.truncate(MAX_LISTED_ENTRIES);
        Ok(listed)
    }

    /// Collects recognisable files below `start` without recursing into the service.
    async fn collect_files(
        &self,
        start: PathBuf,
        root: &Path,
        extensions: &[&str],
    ) -> CoreResult<Vec<WorkspaceEntry>> {
        let mut found = Vec::new();
        let mut stack = vec![(start, 0usize)];
        while let Some((directory, depth)) = stack.pop() {
            let entries = self.files.list(&directory).await?;
            for entry in entries {
                if entry.is_dir {
                    if depth + 1 < MAX_WALK_DEPTH {
                        stack.push((entry.path, depth + 1));
                    }
                    continue;
                }
                if !has_extension(&entry.path, extensions) {
                    continue;
                }
                found.push(WorkspaceEntry {
                    name: entry.name,
                    path: relative_of(root, &entry.path),
                    is_dir: false,
                    size_bytes: entry.size_bytes,
                    modified_ms: entry.modified_ms,
                });
                if found.len() >= MAX_LISTED_ENTRIES {
                    return Ok(found);
                }
            }
        }
        Ok(found)
    }

    /// Adds up the size of one directory tree.
    async fn measure(&self, directory: &Path) -> (u64, bool) {
        let mut total = 0u64;
        let mut counted = 0usize;
        let mut stack = vec![(directory.to_path_buf(), 0usize)];
        while let Some((current, depth)) = stack.pop() {
            let Ok(entries) = self.files.list(&current).await else {
                return (total, false);
            };
            for entry in entries {
                if entry.is_dir {
                    if depth + 1 >= MAX_WALK_DEPTH {
                        return (total, false);
                    }
                    stack.push((entry.path, depth + 1));
                } else {
                    total = total.saturating_add(entry.size_bytes);
                    counted += 1;
                    if counted >= MAX_WALK_FILES {
                        return (total, false);
                    }
                }
            }
        }
        (total, true)
    }

    /// The instance game directory, canonical for checking and friendly for showing.
    async fn roots(&self, instance_id: &str) -> CoreResult<(PathBuf, PathBuf)> {
        validate_slug(instance_id)?;
        self.instances.record(instance_id).await?;
        let lexical = self.paths.instance_game_dir(instance_id);
        let canonical = self.files.canonicalize(&lexical).await.ok_or_else(|| {
            CoreError::unavailable(format!(
                "instance `{instance_id}` has no game directory yet"
            ))
        })?;
        Ok((canonical, lexical))
    }

    /// Proves `relative` names something inside the instance and returns it.
    async fn locate(&self, instance_id: &str, relative: &str) -> CoreResult<Located> {
        let (root, lexical_root) = self.roots(instance_id).await?;
        let parts = relative_components(relative)?;
        let mut path = root.clone();
        let mut desktop_path = lexical_root;
        for part in &parts {
            path = path.join(part);
            desktop_path = desktop_path.join(part);
        }
        let canonical = self.files.canonicalize(&path).await.ok_or_else(|| {
            CoreError::unavailable(format!(
                "`{}` does not exist in this instance",
                relative.trim()
            ))
        })?;
        if !canonical.starts_with(&root) {
            return Err(CoreError::invalid_request(
                "that path leaves the instance directory",
            ));
        }
        Ok(Located {
            root,
            path: canonical,
            desktop_path,
        })
    }
}

/// Splits a UI path into the components below the instance root.
///
/// An empty result means the root itself. Anything that could step outside it is
/// refused before a single filesystem call.
fn relative_components(value: &str) -> CoreResult<Vec<String>> {
    if value.len() > MAX_RELATIVE_LENGTH {
        return Err(CoreError::invalid_request("the path is too long"));
    }
    if value.chars().any(char::is_control) {
        return Err(CoreError::invalid_request(
            "the path contains control characters",
        ));
    }
    let trimmed = value.trim();
    if trimmed.starts_with('/') || trimmed.starts_with('\\') {
        return Err(CoreError::invalid_request(
            "the path must be inside the instance directory",
        ));
    }
    let mut parts = Vec::new();
    for raw in trimmed.split(['/', '\\']) {
        let part = raw.trim();
        if part.is_empty() {
            continue;
        }
        if part == "." || part == ".." || part.contains(':') {
            return Err(CoreError::invalid_request(format!(
                "`{part}` cannot be used inside an instance path"
            )));
        }
        if part.len() > 255 {
            return Err(CoreError::invalid_request("a file name is too long"));
        }
        parts.push(part.to_owned());
    }
    Ok(parts)
}

fn workspace_entry(root: &Path, entry: &DirEntry) -> WorkspaceEntry {
    WorkspaceEntry {
        name: entry.name.clone(),
        path: relative_of(root, &entry.path),
        is_dir: entry.is_dir,
        size_bytes: entry.size_bytes,
        modified_ms: entry.modified_ms,
    }
}

/// Rewrites an absolute path as the instance-relative path the frontend sends back.
fn relative_of(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|relative| {
            relative
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/")
        })
        .unwrap_or_default()
}

fn has_extension(path: &Path, allowed: &[&str]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            allowed
                .iter()
                .any(|wanted| extension.eq_ignore_ascii_case(wanted))
        })
}

/// Decodes the end of a file, dropping the partial first line a tail read starts with.
fn tail_text(bytes: &[u8], truncated: bool) -> String {
    let text = String::from_utf8_lossy(bytes).into_owned();
    if !truncated {
        return text;
    }
    match text.find('\n') {
        Some(index) if index + 1 < text.len() => text[index + 1..].to_owned(),
        _ => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{InstanceRepository, SystemClock};
    use async_trait::async_trait;
    use mnelys_domain::{InstanceRecord, InstanceSpec};
    use std::collections::{BTreeMap, BTreeSet};
    use std::sync::Mutex;

    const ROOT: &str = "/data/instances/main/game";

    /// A filesystem the tests own completely: files, directories and the symlinks a
    /// hostile instance could plant.
    #[derive(Default)]
    struct FakeFiles {
        bodies: BTreeMap<PathBuf, Vec<u8>>,
        directories: BTreeSet<PathBuf>,
        /// Where a symlink inside the instance points.
        links: BTreeMap<PathBuf, PathBuf>,
        deleted: Mutex<Vec<PathBuf>>,
    }

    impl FakeFiles {
        fn with_root() -> Self {
            Self {
                directories: [PathBuf::from(ROOT)].into_iter().collect(),
                ..Self::default()
            }
        }

        fn file(mut self, relative: &str, body: &str) -> Self {
            let path = PathBuf::from(ROOT).join(relative);
            self.bodies.insert(path, body.as_bytes().to_vec());
            self
        }

        fn directory(mut self, relative: &str) -> Self {
            self.directories.insert(PathBuf::from(ROOT).join(relative));
            self
        }

        fn deletions(&self) -> Vec<PathBuf> {
            self.deleted.lock().expect("lock").iter().cloned().collect()
        }
    }

    #[async_trait]
    impl FileStore for FakeFiles {
        async fn read(&self, path: &Path) -> CoreResult<Vec<u8>> {
            self.bodies
                .get(path)
                .cloned()
                .ok_or_else(|| CoreError::io("missing"))
        }

        async fn read_tail(&self, path: &Path, max_bytes: u64) -> CoreResult<Vec<u8>> {
            let body = self
                .bodies
                .get(path)
                .cloned()
                .ok_or_else(|| CoreError::io("missing"))?;
            let window = usize::try_from(max_bytes).unwrap_or(usize::MAX);
            Ok(body[body.len().saturating_sub(window)..].to_vec())
        }

        async fn write_atomic(&self, _: &Path, _: &[u8]) -> CoreResult<()> {
            Ok(())
        }

        async fn exists(&self, path: &Path) -> bool {
            self.bodies.contains_key(path) || self.directories.contains(path)
        }

        async fn is_directory(&self, path: &Path) -> bool {
            self.directories.contains(path)
        }

        async fn remove(&self, path: &Path) -> CoreResult<()> {
            self.deleted.lock().expect("lock").push(path.to_path_buf());
            Ok(())
        }

        async fn remove_dir_all(&self, path: &Path) -> CoreResult<()> {
            self.deleted.lock().expect("lock").push(path.to_path_buf());
            Ok(())
        }

        async fn create_dir_all(&self, _: &Path) -> CoreResult<()> {
            Ok(())
        }

        async fn list(&self, path: &Path) -> CoreResult<Vec<DirEntry>> {
            let mut children: BTreeMap<String, (bool, u64)> = BTreeMap::new();
            for (key, body) in &self.bodies {
                if let Ok(relative) = key.strip_prefix(path) {
                    if let Some(first) = relative.components().next() {
                        let name = first.as_os_str().to_string_lossy().into_owned();
                        let is_dir = relative.components().count() > 1;
                        let size = u64::try_from(body.len()).unwrap_or(u64::MAX);
                        children.insert(name, (is_dir, if is_dir { 0 } else { size }));
                    }
                }
            }
            for directory in &self.directories {
                if let Ok(relative) = directory.strip_prefix(path) {
                    if let Some(first) = relative.components().next() {
                        children
                            .insert(first.as_os_str().to_string_lossy().into_owned(), (true, 0));
                    }
                }
            }
            Ok(children
                .into_iter()
                .map(|(name, (is_dir, size_bytes))| DirEntry {
                    path: path.join(&name),
                    name,
                    is_dir,
                    size_bytes,
                    modified_ms: None,
                })
                .collect())
        }

        async fn len(&self, path: &Path) -> Option<u64> {
            self.bodies
                .get(path)
                .map(|body| u64::try_from(body.len()).unwrap_or(u64::MAX))
        }

        async fn copy(&self, _: &Path, _: &Path) -> CoreResult<()> {
            Ok(())
        }

        async fn canonicalize(&self, path: &Path) -> Option<PathBuf> {
            if let Some(target) = self.links.get(path) {
                return Some(target.clone());
            }
            if self.bodies.contains_key(path) || self.directories.contains(path) {
                return Some(path.to_path_buf());
            }
            // Below a registered directory everything exists: the fake has no notion of
            // a missing intermediate.
            let inside = path
                .ancestors()
                .skip(1)
                .any(|ancestor| self.directories.contains(ancestor));
            inside.then(|| path.to_path_buf())
        }

        async fn extract_zip(&self, _: &Path, _: &Path, _: &[String]) -> CoreResult<usize> {
            Ok(0)
        }
    }

    #[derive(Default)]
    struct FakeRepo {
        records: BTreeMap<String, InstanceRecord>,
    }

    #[async_trait]
    impl InstanceRepository for FakeRepo {
        async fn list(&self) -> CoreResult<Vec<InstanceRecord>> {
            Ok(self.records.values().cloned().collect())
        }

        async fn get(&self, id: &str) -> CoreResult<Option<InstanceRecord>> {
            Ok(self.records.get(id).cloned())
        }

        async fn save(&self, _: &InstanceRecord) -> CoreResult<()> {
            Ok(())
        }

        async fn delete(&self, _: &str) -> CoreResult<()> {
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeRevealer {
        shown: Mutex<Vec<PathBuf>>,
    }

    impl EntryRevealer for FakeRevealer {
        fn reveal(&self, path: &Path) -> CoreResult<()> {
            self.shown.lock().expect("lock").push(path.to_path_buf());
            Ok(())
        }
    }

    fn instance_record() -> InstanceRecord {
        InstanceRecord::new(
            InstanceSpec {
                id: String::from("main"),
                name: String::from("Main"),
                game_version: String::from("1.21.4"),
                loader: mnelys_domain::LoaderKind::Vanilla,
                loader_version: None,
            },
            1,
            String::from("1.21.4"),
        )
    }

    struct Harness {
        files: Arc<FakeFiles>,
        revealer: Arc<FakeRevealer>,
        service: WorkspaceService,
    }

    fn harness(files: FakeFiles) -> Harness {
        let paths = GamePaths::new("/data");
        let files: Arc<FakeFiles> = Arc::new(files);
        let store: Arc<dyn FileStore> = files.clone();
        let revealer: Arc<FakeRevealer> = Arc::new(FakeRevealer::default());
        let show: Arc<dyn EntryRevealer> = revealer.clone();
        let mut repo = FakeRepo::default();
        repo.records.insert(String::from("main"), instance_record());
        let instances = Arc::new(InstanceService::new(
            Arc::new(repo) as Arc<dyn InstanceRepository>,
            Arc::clone(&store),
            paths.clone(),
            Arc::new(SystemClock),
        ));
        Harness {
            service: WorkspaceService::new(store, paths, instances, show),
            files,
            revealer,
        }
    }

    #[test]
    fn paths_that_could_leave_the_instance_are_refused_before_io() {
        for value in [
            "../outside",
            "mods/../../outside",
            "mods\\..\\..\\outside",
            "/etc/passwd",
            "\\\\server\\share",
            "C:/Windows/win.ini",
            "mods/:weird",
            "logs\0latest.log",
        ] {
            assert!(
                relative_components(value).is_err(),
                "`{value}` should be refused"
            );
        }
    }

    #[test]
    fn ordinary_paths_become_components() {
        assert_eq!(relative_components("").expect("root"), Vec::<String>::new());
        assert_eq!(
            relative_components("logs/").expect("logs"),
            vec![String::from("logs")]
        );
        assert_eq!(
            relative_components("schematics\\builds\\house.litem").expect("nested"),
            vec![
                String::from("schematics"),
                String::from("builds"),
                String::from("house.litem")
            ]
        );
    }

    #[tokio::test]
    async fn a_symlink_out_of_the_instance_is_refused() {
        let mut files = FakeFiles::with_root();
        files.links.insert(
            PathBuf::from(ROOT).join("logs"),
            PathBuf::from("/elsewhere/secrets"),
        );
        let harness = harness(files);
        let error = harness
            .service
            .list("main", "logs")
            .await
            .expect_err("a symlink out must fail");
        assert_eq!(error.code, mnelys_domain::CoreErrorCode::InvalidRequest);
    }

    #[tokio::test]
    async fn an_unknown_instance_is_refused() {
        let harness = harness(FakeFiles::with_root());
        assert!(harness.service.list("ghost", "").await.is_err());
        let traversal = harness
            .service
            .delete("..", "mods/x.jar")
            .await
            .expect_err("a traversal id must fail");
        assert_eq!(traversal.code, mnelys_domain::CoreErrorCode::InvalidRequest);
    }

    #[tokio::test]
    async fn the_instance_root_itself_is_not_deletable() {
        let harness = harness(FakeFiles::with_root());
        let error = harness
            .service
            .delete("main", "")
            .await
            .expect_err("the root stays");
        assert!(error
            .message
            .contains("instance directory itself is not deleted"));
    }

    #[tokio::test]
    async fn a_listing_puts_directories_first_and_names_the_directory() {
        let files = FakeFiles::with_root()
            .directory("saves")
            .file("saves/overworld/level.dat", "data")
            .file("mods/a-mod.jar", "jar")
            .file("mods/zoo.jar", "j");
        let harness = harness(files);
        let listing = harness.service.list("main", "mods").await.expect("list");
        assert_eq!(listing.path, "mods");
        assert_eq!(
            listing
                .entries
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            vec!["mods/a-mod.jar", "mods/zoo.jar"]
        );
        assert_eq!(listing.entries[0].size_bytes, 3);
    }

    #[tokio::test]
    async fn a_directory_is_deleted_recursively_and_a_file_is_deleted_plainly() {
        let files = FakeFiles::with_root()
            .directory("saves/overworld")
            .file("saves/overworld/level.dat", "data")
            .file("mods/a-mod.jar", "jar");
        let harness = harness(files);
        harness
            .service
            .delete("main", "saves/overworld")
            .await
            .expect("delete world");
        harness
            .service
            .delete("main", "mods/a-mod.jar")
            .await
            .expect("delete mod");
        let deleted = harness.files.deletions();
        assert_eq!(
            deleted,
            vec![
                PathBuf::from(ROOT).join("saves/overworld"),
                PathBuf::from(ROOT).join("mods/a-mod.jar")
            ]
        );
    }

    #[tokio::test]
    async fn only_log_and_text_files_are_readable() {
        let files = FakeFiles::with_root()
            .file("logs/latest.log", "first\nsecond\n")
            .file("mods/a-mod.jar", "jar");
        let harness = harness(files);
        let tail = harness
            .service
            .read_log("main", "logs/latest.log")
            .await
            .expect("read");
        assert_eq!(tail.text, "first\nsecond\n");
        assert!(!tail.truncated);
        assert_eq!(tail.path, "logs/latest.log");
        let error = harness
            .service
            .read_log("main", "mods/a-mod.jar")
            .await
            .expect_err("a jar is not a log");
        assert!(error.message.contains("only log and text files"));
    }

    #[test]
    fn a_truncated_tail_starts_on_a_complete_line() {
        assert_eq!(tail_text(b"34\nline\n", true), "line\n");
        assert_eq!(tail_text(b"whole file\n", false), "whole file\n");
        assert_eq!(tail_text(b"", true), "");
    }

    #[tokio::test]
    async fn logs_and_crash_reports_are_listed_apart() {
        let files = FakeFiles::with_root()
            .directory("logs")
            .directory("crash-reports")
            .file("logs/latest.log", "a")
            .file("logs/debug.log", "bb")
            .file("logs/2026-01-01-1.log.gz", "zip")
            .file("crash-reports/crash.txt", "boom");
        let harness = harness(files);
        let listing = harness.service.logs("main").await.expect("logs");
        assert_eq!(
            listing
                .logs
                .iter()
                .map(|entry| entry.name.as_str())
                .collect::<Vec<_>>(),
            vec!["debug.log", "latest.log"]
        );
        assert_eq!(listing.crash_reports.len(), 1);
        assert_eq!(listing.crash_reports[0].size_bytes, 4);
    }

    #[tokio::test]
    async fn worlds_report_the_size_the_walk_actually_measured() {
        let files = FakeFiles::with_root()
            .directory("saves/overworld")
            .file("saves/overworld/level.dat", "12345")
            .directory("saves/overworld/region")
            .file("saves/overworld/region/r.0.0.mca", "67890")
            .file("saves/not-a-world.txt", "x");
        let harness = harness(files);
        let worlds = harness.service.worlds("main").await.expect("worlds");
        assert_eq!(worlds.len(), 1);
        assert_eq!(worlds[0].name, "overworld");
        assert_eq!(worlds[0].path, "saves/overworld");
        assert_eq!(worlds[0].size_bytes, 10);
        assert!(worlds[0].size_complete);
    }

    #[tokio::test]
    async fn schematics_are_found_in_subfolders_too() {
        let files = FakeFiles::with_root()
            .directory("schematics/builds")
            .file("schematics/house.litem", "litematica")
            .file("schematics/builds/tower.schem", "worldedit")
            .file("schematics/builds/notes.md", "text");
        let harness = harness(files);
        let found = harness
            .service
            .schematics("main")
            .await
            .expect("schematics");
        assert_eq!(
            found
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            vec!["schematics/builds/tower.schem", "schematics/house.litem"]
        );
    }

    #[tokio::test]
    async fn revealing_uses_the_plain_instance_path() {
        let files = FakeFiles::with_root().file("mods/a-mod.jar", "jar");
        let harness = harness(files);
        harness
            .service
            .reveal("main", "mods/a-mod.jar")
            .await
            .expect("reveal");
        assert_eq!(
            harness.revealer.shown.lock().expect("lock")[0],
            PathBuf::from(ROOT).join("mods/a-mod.jar")
        );
    }
}
