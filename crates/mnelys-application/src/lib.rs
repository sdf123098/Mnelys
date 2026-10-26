//! Application layer: use-cases plus the ports they need.
//!
//! Nothing here knows about Tauri, the filesystem or a concrete storage engine. The
//! composition root supplies implementations; presentation only calls these services.

pub mod accounts;
pub mod content;
pub mod install;
pub mod instances;
pub mod launch;
pub mod logs;
pub mod metadata;
pub mod mirrors;
pub mod paths;
pub mod ports;
pub mod settings;
pub mod tasks;
pub mod util;
pub mod workspace;

pub use accounts::{AccountService, MicrosoftConfig};
pub use content::{
    ContentInstallReport, ContentManifest, ContentRegistry, ContentService, ExportedInstance,
    InstalledContent, InstanceExport,
};
pub use install::{InstallReport, InstallService};
pub use instances::{InstanceService, InstanceSummary, UpdateInstance};
pub use launch::{LaunchOptions, LaunchService};
pub use logs::{LogEntry, LogLevel, LogService, LogStore};
pub use metadata::MetadataService;
pub use mirrors::{MirrorKind, MirrorPolicy, MirrorPreference, MirrorProbe};
pub use paths::GamePaths;
pub use ports::{
    default_task_label, fetch_bytes, fetch_json, request_json, Clock, ContentProvider,
    ContentSearchRequest, ContentVersionFilter, DirEntry, EntryRevealer, FileStore, GameRunner,
    HttpClient, HttpMethod, HttpRequest, HttpResponse, JavaProvider, LoaderInstallContext,
    LoaderProvider, NullReporter, SecretStore, SystemClock,
};
pub use settings::{AppSettings, SettingsStore};
pub use tasks::{TaskHandle, TaskRegistry, TaskReporter};
pub use workspace::{
    FileListing, LogListing, LogTail, WorkspaceEntry, WorkspaceService, WorldEntry,
};
