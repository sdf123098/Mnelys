//! Infrastructure adapters.
//!
//! Only adapters live here. They depend on the application ports, never the other way
//! around, so the core stays testable without a webview or a real filesystem.

pub mod accounts;
pub mod curseforge;
pub mod fabric;
pub mod files;
pub mod forge;
pub mod http;
pub mod instances;
pub mod java;
pub mod layout;
pub mod loaders;
pub mod memory_log;
pub mod modrinth;
pub mod process;
pub mod reveal;
pub mod secrets;
pub mod settings;

pub use accounts::JsonAccountRepository;
pub use curseforge::CurseForgeProvider;
pub use fabric::FabricLikeProvider;
pub use files::LocalFileStore;
pub use forge::InstallerLoaderProvider;
pub use http::{ReqwestHttp, USER_AGENT};
pub use instances::JsonInstanceRepository;
pub use java::LocalJavaProvider;
pub use layout::{default_root, HOME_ENV};
pub use loaders::MetaLoaderProvider;
pub use memory_log::{MemoryLogStore, DEFAULT_LOG_CAPACITY};
pub use modrinth::ModrinthProvider;
pub use process::TokioGameRunner;
pub use reveal::DesktopRevealer;
pub use secrets::KeyringSecretStore;
pub use settings::JsonSettingsStore;
