//! Mnelys domain vocabulary.
//!
//! Everything here is pure data and pure functions: no filesystem, no network, no
//! Tauri. That is what lets the install and launch plans be tested in milliseconds.

pub mod account;
pub mod content;
pub mod download;
pub mod error;
pub mod instance;
pub mod java;
pub mod launch;
pub mod loader;
pub mod task;
pub mod version;

pub use account::{
    AccountKind, AccountRecord, AccountSummary, AuthSession, DeviceCodeChallenge, YggdrasilEndpoint,
};
pub use content::{
    ContentDependency, ContentFile, ContentKind, ContentProject, ContentProviderId,
    ContentSearchPage, ContentVersion, DependencyKind,
};
pub use download::{DownloadItem, DownloadKind, DownloadPlan};
pub use error::{CoreError, CoreErrorCode, CoreResult};
pub use instance::{
    offline_uuid, validate_offline_username, validate_slug, InstanceRecord, InstanceSpec,
    LoaderKind,
};
pub use java::{JavaRuntime, JavaSource};
pub use launch::{
    classpath_separator, render_argument_list, render_arguments, LaunchPlan, LaunchSession,
    LaunchState,
};
pub use loader::LoaderVersionInfo;
pub use task::{TaskKind, TaskProgress, TaskSnapshot, TaskState};
pub use version::{
    rules_allow, AssetIndex, AssetIndexDocument, AssetObject, DownloadInfo, Library, LoggingClient,
    LoggingConfig, MavenCoordinate, Platform, VersionDocument, VersionManifest,
};
