//! The local filesystem adapter.
//!
//! Writes go through a temporary file in the destination directory followed by a
//! rename, so an interrupted download never leaves a truncated jar that looks
//! installed. Zip extraction runs on the blocking pool because the `zip` crate is
//! synchronous.

use std::io::{self, SeekFrom};
use std::path::{Component, Path, PathBuf};

use async_trait::async_trait;
use mnelys_application::ports::{DirEntry, FileStore};
use mnelys_domain::{CoreError, CoreResult};
use tokio::io::{AsyncReadExt, AsyncSeekExt};

/// A [`FileStore`] rooted in the real filesystem.
#[derive(Debug, Clone, Copy, Default)]
pub struct LocalFileStore;

impl LocalFileStore {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

fn io_error(action: &str, path: &Path, error: &io::Error) -> CoreError {
    CoreError::io(format!("could not {action} {}", path.display())).with_details(error.to_string())
}

fn modified_ms(metadata: &std::fs::Metadata) -> Option<u64> {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX))
}

#[async_trait]
impl FileStore for LocalFileStore {
    async fn read(&self, path: &Path) -> CoreResult<Vec<u8>> {
        tokio::fs::read(path)
            .await
            .map_err(|error| io_error("read", path, &error))
    }

    async fn read_tail(&self, path: &Path, max_bytes: u64) -> CoreResult<Vec<u8>> {
        let mut file = tokio::fs::File::open(path)
            .await
            .map_err(|error| io_error("read", path, &error))?;
        let metadata = file
            .metadata()
            .await
            .map_err(|error| io_error("read", path, &error))?;
        let size = metadata.len();
        let skip = size.saturating_sub(max_bytes);
        if skip > 0 {
            file.seek(SeekFrom::Start(skip))
                .await
                .map_err(|error| io_error("read", path, &error))?;
        }
        let mut buffer =
            Vec::with_capacity(usize::try_from((size - skip).min(1024 * 1024)).unwrap_or(0));
        file.read_to_end(&mut buffer)
            .await
            .map_err(|error| io_error("read", path, &error))?;
        Ok(buffer)
    }

    async fn write_atomic(&self, path: &Path, data: &[u8]) -> CoreResult<()> {
        let parent = path.parent().ok_or_else(|| {
            CoreError::invalid_request(format!("{} has no parent directory", path.display()))
        })?;
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| io_error("create", parent, &error))?;

        let temporary = temporary_path(path);
        tokio::fs::write(&temporary, data)
            .await
            .map_err(|error| io_error("write", &temporary, &error))?;
        if let Err(error) = tokio::fs::rename(&temporary, path).await {
            let _ = tokio::fs::remove_file(&temporary).await;
            return Err(io_error("replace", path, &error));
        }
        Ok(())
    }

    async fn exists(&self, path: &Path) -> bool {
        tokio::fs::try_exists(path).await.unwrap_or(false)
    }

    async fn is_directory(&self, path: &Path) -> bool {
        tokio::fs::metadata(path)
            .await
            .map(|metadata| metadata.is_dir())
            .unwrap_or(false)
    }

    async fn remove(&self, path: &Path) -> CoreResult<()> {
        match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error("remove", path, &error)),
        }
    }

    async fn remove_dir_all(&self, path: &Path) -> CoreResult<()> {
        match tokio::fs::remove_dir_all(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(io_error("remove", path, &error)),
        }
    }

    async fn create_dir_all(&self, path: &Path) -> CoreResult<()> {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|error| io_error("create", path, &error))
    }

    async fn list(&self, path: &Path) -> CoreResult<Vec<DirEntry>> {
        let mut entries = Vec::new();
        let mut reader = match tokio::fs::read_dir(path).await {
            Ok(reader) => reader,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(entries),
            Err(error) => return Err(io_error("list", path, &error)),
        };
        while let Some(entry) = reader
            .next_entry()
            .await
            .map_err(|error| io_error("list", path, &error))?
        {
            let entry_path = entry.path();
            let metadata = entry.metadata().await.ok();
            let is_dir = match &metadata {
                Some(info) => info.is_dir(),
                None => matches!(entry.file_type().await, Ok(kind) if kind.is_dir()),
            };
            let size_bytes = metadata
                .as_ref()
                .filter(|info| !info.is_dir())
                .map_or(0, std::fs::Metadata::len);
            let modified = metadata.as_ref().and_then(modified_ms);
            entries.push(DirEntry {
                name: entry.file_name().to_string_lossy().into_owned(),
                path: entry_path,
                is_dir,
                size_bytes,
                modified_ms: modified,
            });
        }
        entries.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(entries)
    }

    async fn len(&self, path: &Path) -> Option<u64> {
        tokio::fs::metadata(path)
            .await
            .ok()
            .filter(|metadata| metadata.is_file())
            .map(|metadata| metadata.len())
    }

    async fn copy(&self, from: &Path, to: &Path) -> CoreResult<()> {
        if let Some(parent) = to.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| io_error("create", parent, &error))?;
        }
        tokio::fs::copy(from, to)
            .await
            .map_err(|error| io_error("copy", from, &error))?;
        Ok(())
    }

    async fn canonicalize(&self, path: &Path) -> Option<PathBuf> {
        tokio::fs::canonicalize(path).await.ok()
    }

    async fn extract_zip(
        &self,
        archive: &Path,
        destination: &Path,
        excluded_prefixes: &[String],
    ) -> CoreResult<usize> {
        let archive = archive.to_path_buf();
        let destination = destination.to_path_buf();
        let excluded: Vec<String> = excluded_prefixes
            .iter()
            .map(|prefix| prefix.replace('\\', "/"))
            .collect();

        tokio::task::spawn_blocking(move || extract(archive, destination, &excluded))
            .await
            .map_err(|error| {
                CoreError::internal("the archive extraction task failed")
                    .with_details(error.to_string())
            })?
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("download"));
    let unique = format!(
        ".{}.{}.{}.tmp",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.subsec_nanos())
            .unwrap_or(0)
    );
    path.with_file_name(unique)
}

fn extract(archive: PathBuf, destination: PathBuf, excluded: &[String]) -> CoreResult<usize> {
    let file = std::fs::File::open(&archive).map_err(|error| io_error("open", &archive, &error))?;
    let mut zip = zip::ZipArchive::new(file).map_err(|error| {
        CoreError::io(format!("could not read the archive {}", archive.display()))
            .with_details(error.to_string())
    })?;
    std::fs::create_dir_all(&destination)
        .map_err(|error| io_error("create", &destination, &error))?;

    let mut written = 0usize;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|error| {
            CoreError::io(format!("entry {index} of {} is damaged", archive.display()))
                .with_details(error.to_string())
        })?;
        let name = entry.name().replace('\\', "/");
        if excluded
            .iter()
            .any(|prefix| name.starts_with(prefix.as_str()))
        {
            continue;
        }
        let Some(relative) = entry.enclosed_name() else {
            continue;
        };
        if !is_safe_relative(&relative) {
            continue;
        }
        let target = destination.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&target)
                .map_err(|error| io_error("create", &target, &error))?;
            continue;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io_error("create", parent, &error))?;
        }
        let mut output =
            std::fs::File::create(&target).map_err(|error| io_error("create", &target, &error))?;
        io::copy(&mut entry, &mut output).map_err(|error| io_error("extract", &target, &error))?;
        written += 1;
    }
    Ok(written)
}

fn is_safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mnelys-files-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|elapsed| elapsed.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        dir
    }

    #[tokio::test]
    async fn a_write_is_visible_after_it_completes() {
        let dir = temp_dir("write");
        let path = dir.join("nested/value.txt");
        let store = LocalFileStore::new();
        store.write_atomic(&path, b"hello").await.expect("write");
        assert_eq!(store.read(&path).await.expect("read"), b"hello");
        assert_eq!(store.len(&path).await, Some(5));
        std::fs::remove_dir_all(dir).expect("cleanup");
    }

    #[tokio::test]
    async fn removing_a_missing_file_is_not_an_error() {
        let store = LocalFileStore::new();
        store
            .remove(Path::new("definitely-missing-mnelys-file"))
            .await
            .expect("idempotent remove");
    }

    #[tokio::test]
    async fn extraction_skips_excluded_prefixes() {
        let dir = temp_dir("zip");
        let archive_path = dir.join("archive.zip");
        let file = std::fs::File::create(&archive_path).expect("create zip");
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        writer
            .start_file("META-INF/MANIFEST.MF", options)
            .expect("entry");
        writer.write_all(b"manifest").expect("write");
        writer.start_file("data/hello.txt", options).expect("entry");
        writer.write_all(b"hello").expect("write");
        writer.finish().expect("finish");

        let store = LocalFileStore::new();
        let out = dir.join("out");
        let written = store
            .extract_zip(&archive_path, &out, &[String::from("META-INF/")])
            .await
            .expect("extract");
        assert_eq!(written, 1);
        assert!(out.join("data/hello.txt").exists());
        assert!(!out.join("META-INF").exists());
        std::fs::remove_dir_all(dir).expect("cleanup");
    }

    #[test]
    fn only_plain_relative_paths_are_safe() {
        assert!(is_safe_relative(Path::new("a/b.txt")));
        assert!(!is_safe_relative(Path::new("../escape")));
        assert!(!is_safe_relative(Path::new("/absolute")));
    }
}
