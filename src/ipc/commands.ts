/**
 * The typed Rust command surface.
 *
 * ADR 0005 requires the Rust/frontend boundary to be typed and versioned; ADR 0006
 * makes it a single contract with exactly one client. This file is the TypeScript
 * half of the contract implemented in `crates/mnelys-presentation/src/commands/`.
 *
 * `CONTRACT_VERSION` must equal the `contractVersion` the core reports through
 * `core_version`. `src-tauri/tests/contract.rs` checks that the command names declared
 * here match the ones the Rust side registers.
 */

export const CONTRACT_VERSION = 4;

/** Result of `core_version` — the start-up handshake that keeps both halves in step. */
export interface CoreVersion {
  readonly version: string;
  readonly contractVersion: number;
  readonly profile: 'debug' | 'release';
}

/** Result of `runtime_info` — host facts the UI adapts to. */
export interface RuntimeInfo {
  readonly os: string;
  readonly arch: string;
  readonly webview: string;
  readonly family: 'webview2' | 'wkwebview' | 'webkitgtk' | 'unknown';
}

/** Severity of one core log entry, mirroring `mnelys_application::LogLevel`. */
export type LogLevel = 'trace' | 'debug' | 'info' | 'warn' | 'error';

/** Result item of `app_logs`. Timestamps stay numeric; formatting is the UI's job. */
export interface AppLogEntry {
  readonly id: number;
  readonly timestampMs: number;
  readonly level: LogLevel;
  readonly target: string;
  readonly message: string;
}

/** Mod loaders, matching `mnelys_domain::LoaderKind` wire names. */
export type LoaderKind = 'vanilla' | 'fabric' | 'quilt' | 'legacy-fabric' | 'forge' | 'neoforge';

/** Content platforms, matching `mnelys_domain::ContentProviderId`. */
export type ContentProviderId = 'modrinth' | 'curseforge';

/** Content categories, matching `mnelys_domain::ContentKind`. */
export type ContentKind = 'mod' | 'resourcepack' | 'shader' | 'world' | 'datapack';

/** Mojang versus BMCLAPI versus automatic selection. */
export type MirrorPreference = 'direct' | 'bmclapi' | 'auto';

/** The mirror actually in use. */
export type MirrorKind = 'direct' | 'bmclapi';

/** Result of `get_settings` / `update_settings`. */
export interface SettingsView {
  readonly mirror: MirrorPreference;
  readonly activeMirror: MirrorKind;
  readonly memoryMb: number;
  readonly root: string;
}

/** Partial settings update; omitted fields keep their stored value. */
export interface SettingsUpdate {
  readonly mirror?: MirrorPreference;
  readonly memoryMb?: number;
}

/** Result of `probe_mirrors`. */
export interface MirrorProbe {
  readonly mirror: MirrorKind;
  readonly reachable: boolean;
  readonly latencyMs?: number;
}

/** One release in Mojang's version manifest. Field names mirror the upstream JSON. */
export interface VersionEntry {
  readonly id: string;
  readonly type: 'release' | 'snapshot' | 'old_beta' | 'old_alpha';
  readonly url: string;
  readonly sha1?: string;
  readonly releaseTime?: string;
}

/** Result of `list_versions`. */
export interface VersionManifest {
  readonly latest: { readonly release: string; readonly snapshot: string };
  readonly versions: readonly VersionEntry[];
}

/** Result item of `list_loaders`. */
export interface LoaderVersionInfo {
  readonly version: string;
  readonly gameVersion: string;
  readonly loader: LoaderKind;
  readonly stable: boolean;
}

/** Result item of `list_java`. */
export interface JavaRuntime {
  readonly path: string;
  readonly major: number;
  readonly version: string;
  readonly vendor: string;
  readonly source: 'managed' | 'system';
}

/** One instance row. */
export interface InstanceSummary {
  readonly id: string;
  readonly name: string;
  readonly gameVersion: string;
  readonly loader: LoaderKind;
  readonly loaderVersion?: string;
  readonly versionId: string;
  readonly createdAtMs: number;
  readonly lastLaunchedMs?: number;
  readonly javaPath?: string;
  readonly installed: boolean;
  readonly directory: string;
}

/** What creating an instance sends. */
export interface InstanceSpec {
  readonly id: string;
  readonly name: string;
  readonly gameVersion: string;
  readonly loader?: LoaderKind;
  readonly loaderVersion?: string;
}

/** What editing an instance may change. An empty `javaPath` clears the override. */
export interface UpdateInstance {
  readonly name?: string;
  readonly javaPath?: string;
}

/** One spawned game process. */
export interface LaunchSession {
  readonly id: string;
  readonly instanceId: string;
  readonly versionId: string;
  readonly pid: number;
  readonly state: 'running' | 'exited' | 'failed' | 'stopped';
  readonly startedAtMs: number;
  readonly finishedAtMs?: number;
  readonly exitCode?: number;
}

/** Per-launch overrides. */
export interface LaunchOptions {
  readonly accountId?: string;
  readonly memoryMb?: number;
}

/** Where an account's identity comes from. */
export type AccountKind = 'microsoft' | 'offline' | 'yggdrasil';

/** One account row; never contains a token. */
export interface AccountSummary {
  readonly id: string;
  readonly kind: AccountKind;
  readonly name: string;
  readonly uuid: string;
  readonly selected: boolean;
  readonly addedAtMs: number;
  readonly lastUsedMs?: number;
}

/** The device-code challenge the user approves in a browser. */
export interface DeviceCodeChallenge {
  readonly deviceCode: string;
  readonly userCode: string;
  readonly verificationUri: string;
  readonly expiresIn: number;
  readonly interval: number;
  readonly message: string;
}

/** One content project row. */
export interface ContentProject {
  readonly id: string;
  readonly provider: ContentProviderId;
  readonly kind: ContentKind;
  readonly slug: string;
  readonly name: string;
  readonly summary: string;
  readonly iconUrl?: string;
  readonly downloads: number;
  readonly author: string;
  readonly updatedAt?: string;
}

/** One page of content search results. */
export interface ContentSearchPage {
  readonly provider: ContentProviderId;
  readonly hits: readonly ContentProject[];
  readonly offset: number;
  readonly limit: number;
  readonly total: number;
}

/** One downloadable file attached to a version. */
export interface ContentFile {
  readonly url: string;
  readonly filename: string;
  readonly sha1?: string;
  readonly size?: number;
  readonly primary: boolean;
}

/** A dependency edge. */
export interface ContentDependency {
  readonly projectId?: string;
  readonly versionId?: string;
  readonly kind: 'required' | 'optional' | 'incompatible' | 'embedded';
}

/** One published version of a project. */
export interface ContentVersion {
  readonly id: string;
  readonly projectId: string;
  readonly provider: ContentProviderId;
  readonly name: string;
  readonly versionNumber: string;
  readonly gameVersions: readonly string[];
  readonly loaders: readonly string[];
  readonly files: readonly ContentFile[];
  readonly dependencies: readonly ContentDependency[];
}

/** One installed content file, as the instance manifest records it. */
export interface InstalledContent {
  readonly provider: ContentProviderId;
  readonly projectId: string;
  readonly versionId: string;
  readonly name: string;
  readonly filename: string;
  readonly kind: ContentKind;
  readonly installedAtMs: number;
}

/** A saved portable instance manifest. */
export interface ExportedInstance {
  readonly path: string;
  readonly json: string;
}

/**
 * One file or directory below an instance game directory. `path` is slash separated
 * and relative to that directory, which is what the file commands accept back.
 */
export interface WorkspaceEntry {
  readonly name: string;
  readonly path: string;
  readonly isDir: boolean;
  /** Always zero for a directory; measuring one is a separate walk. */
  readonly sizeBytes: number;
  readonly modifiedMs: number | null;
}

/** Result of `list_instance_files`. */
export interface FileListing {
  /** The listed directory; empty for the instance root. */
  readonly path: string;
  readonly entries: readonly WorkspaceEntry[];
  /** True when the directory holds more entries than a listing returns. */
  readonly truncated: boolean;
}

/** Result of `list_instance_logs`. */
export interface LogListing {
  readonly logs: readonly WorkspaceEntry[];
  readonly crashReports: readonly WorkspaceEntry[];
}

/** Result of `read_instance_log`: the end of one log file. */
export interface LogTail {
  readonly path: string;
  readonly text: string;
  /** True when the file is longer than the window the core read. */
  readonly truncated: boolean;
}

/** Result item of `list_instance_worlds`. */
export interface WorldEntry {
  readonly name: string;
  readonly path: string;
  readonly sizeBytes: number;
  /** False when the walk stopped early, which makes the size a lower bound. */
  readonly sizeComplete: boolean;
  readonly modifiedMs: number | null;
}

/** One background task snapshot. */
export interface TaskSnapshot {
  readonly id: string;
  readonly kind:
    | 'metadata'
    | 'download'
    | 'install'
    | 'launch'
    | 'java'
    | 'account'
    | 'content'
    | 'import'
    | 'export';
  readonly state: 'queued' | 'running' | 'succeeded' | 'failed' | 'cancelled';
  readonly label: string;
  readonly progress: {
    readonly current: number;
    readonly total?: number;
    readonly message: string;
  };
  readonly startedAtMs: number;
  readonly finishedAtMs?: number;
  readonly error?: string;
}

/** Arguments for content search. */
export interface SearchContentArgs {
  readonly provider: ContentProviderId;
  readonly query: string;
  readonly kind: ContentKind;
  readonly gameVersion?: string;
  readonly loader?: string;
  readonly offset?: number;
  readonly limit?: number;
}

/** Arguments for a version listing. */
export interface ContentVersionsArgs {
  readonly provider: ContentProviderId;
  readonly projectId: string;
  readonly gameVersion?: string;
  readonly loader?: string;
}

/** Arguments for installing content. */
export interface InstallContentArgs {
  readonly instanceId: string;
  readonly provider: ContentProviderId;
  readonly projectId: string;
  readonly versionId?: string;
  readonly kind: ContentKind;
}

/**
 * Command name to argument and result types. Commands that take no arguments
 * declare `Record<string, never>` so the client keeps one call signature.
 */
export interface CommandContract {
  core_version: { args: Record<string, never>; result: CoreVersion };
  runtime_info: { args: Record<string, never>; result: RuntimeInfo };
  app_logs: { args: { readonly limit?: number }; result: AppLogEntry[] };
  clear_app_logs: { args: Record<string, never>; result: number };

  get_settings: { args: Record<string, never>; result: SettingsView };
  update_settings: { args: { readonly update: SettingsUpdate }; result: SettingsView };
  probe_mirrors: { args: Record<string, never>; result: MirrorProbe[] };

  list_versions: { args: Record<string, never>; result: VersionManifest };
  list_loaders: {
    args: { readonly loader: LoaderKind; readonly gameVersion: string };
    result: LoaderVersionInfo[];
  };

  list_java: { args: Record<string, never>; result: JavaRuntime[] };
  install_java: { args: { readonly major: number }; result: string };

  list_instances: { args: Record<string, never>; result: InstanceSummary[] };
  get_instance: { args: { readonly id: string }; result: InstanceSummary };
  create_instance: { args: { readonly spec: InstanceSpec }; result: InstanceSummary };
  update_instance: {
    args: { readonly id: string; readonly update: UpdateInstance };
    result: InstanceSummary;
  };
  delete_instance: { args: { readonly id: string }; result: null };
  install_instance: { args: { readonly id: string }; result: string };

  launch_instance: {
    args: { readonly id: string; readonly options?: LaunchOptions };
    result: LaunchSession;
  };
  stop_instance: { args: { readonly sessionId: string }; result: boolean };
  launch_sessions: { args: Record<string, never>; result: LaunchSession[] };
  launch_logs: {
    args: { readonly sessionId: string; readonly limit?: number };
    result: string[];
  };

  list_accounts: { args: Record<string, never>; result: AccountSummary[] };
  add_offline_account: {
    args: { readonly args: { readonly username: string } };
    result: AccountSummary;
  };
  add_yggdrasil_account: {
    args: {
      readonly args: {
        readonly authUrl: string;
        readonly username: string;
        readonly password: string;
      };
    };
    result: AccountSummary;
  };
  begin_microsoft_login: { args: Record<string, never>; result: DeviceCodeChallenge };
  complete_microsoft_login: {
    args: { readonly deviceCode: string; readonly interval?: number };
    result: AccountSummary;
  };
  select_account: { args: { readonly id: string }; result: AccountSummary };
  remove_account: { args: { readonly id: string }; result: null };
  refresh_account: { args: { readonly id: string }; result: AccountSummary };

  content_providers: { args: Record<string, never>; result: ContentProviderId[] };
  search_content: { args: { readonly args: SearchContentArgs }; result: ContentSearchPage };
  content_versions: {
    args: { readonly args: ContentVersionsArgs };
    result: ContentVersion[];
  };
  installed_content: { args: { readonly instanceId: string }; result: InstalledContent[] };
  install_content: { args: { readonly args: InstallContentArgs }; result: string };
  remove_content: {
    args: { readonly instanceId: string; readonly projectId: string };
    result: boolean;
  };
  export_instance: { args: { readonly instanceId: string }; result: ExportedInstance };
  import_instance: { args: { readonly args: { readonly json: string } }; result: InstanceSummary };

  list_instance_files: {
    args: { readonly instanceId: string; readonly dir?: string };
    result: FileListing;
  };
  list_instance_logs: { args: { readonly instanceId: string }; result: LogListing };
  read_instance_log: {
    args: { readonly instanceId: string; readonly path: string };
    result: LogTail;
  };
  list_instance_worlds: { args: { readonly instanceId: string }; result: WorldEntry[] };
  list_instance_schematics: {
    args: { readonly instanceId: string };
    result: WorkspaceEntry[];
  };
  delete_instance_entry: {
    args: { readonly instanceId: string; readonly path: string };
    result: null;
  };
  reveal_instance_entry: {
    args: { readonly instanceId: string; readonly path: string };
    result: null;
  };

  list_tasks: { args: Record<string, never>; result: TaskSnapshot[] };
  cancel_task: { args: { readonly id: string }; result: boolean };
}

export type CommandName = keyof CommandContract;
export type CommandArgs<K extends CommandName> = CommandContract[K]['args'];
export type CommandResult<K extends CommandName> = CommandContract[K]['result'];

/** Every command name, as a runtime value, for contract tests and diagnostics. */
export const COMMAND_NAMES = [
  'core_version',
  'runtime_info',
  'app_logs',
  'clear_app_logs',
  'get_settings',
  'update_settings',
  'probe_mirrors',
  'list_versions',
  'list_loaders',
  'list_java',
  'install_java',
  'list_instances',
  'get_instance',
  'create_instance',
  'update_instance',
  'delete_instance',
  'install_instance',
  'launch_instance',
  'stop_instance',
  'launch_sessions',
  'launch_logs',
  'list_accounts',
  'add_offline_account',
  'add_yggdrasil_account',
  'begin_microsoft_login',
  'complete_microsoft_login',
  'select_account',
  'remove_account',
  'refresh_account',
  'content_providers',
  'search_content',
  'content_versions',
  'installed_content',
  'install_content',
  'remove_content',
  'export_instance',
  'import_instance',
  'list_instance_files',
  'list_instance_logs',
  'read_instance_log',
  'list_instance_worlds',
  'list_instance_schematics',
  'delete_instance_entry',
  'reveal_instance_entry',
  'list_tasks',
  'cancel_task',
] as const satisfies readonly CommandName[];
