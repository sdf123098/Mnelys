import { invoke } from '@tauri-apps/api/core';

import type {
  CommandArgs,
  CommandName,
  CommandResult,
  ContentKind,
  ContentProviderId,
  ContentVersionsArgs,
  InstanceSpec,
  InstallContentArgs,
  LaunchOptions,
  LoaderKind,
  SearchContentArgs,
  SettingsUpdate,
  UpdateInstance,
} from './commands';
import { isCoreError, type CoreError, type CoreErrorCode } from './errors';

/**
 * Thrown by every wrapper in this module so callers never have to inspect raw
 * rejection payloads. `code` is stable and safe to branch on.
 */
export class CoreCommandError extends Error {
  readonly code: CoreErrorCode;
  readonly details: string | undefined;

  constructor(error: CoreError) {
    super(error.message);
    this.name = 'CoreCommandError';
    this.code = error.code;
    this.details = error.details;
  }
}

async function call<K extends CommandName>(
  name: K,
  args: CommandArgs<K>,
): Promise<CommandResult<K>> {
  try {
    return await invoke<CommandResult<K>>(name, args);
  } catch (cause) {
    if (isCoreError(cause)) {
      throw new CoreCommandError(cause);
    }
    throw new CoreCommandError({
      code: 'core/internal',
      message:
        typeof cause === 'string' && cause.length > 0
          ? cause
          : 'The Mnelys core failed without returning a structured error.',
    });
  }
}

/**
 * The only door between the frontend and the Rust core (ADR 0006).
 *
 * Components import these named wrappers. Importing `invoke` anywhere else is an
 * ESLint error; importing a Tauri plugin directly is one too, so that the native
 * capability surface stays reviewable (ADR 0005).
 */
export const core = {
  /** Start-up handshake: proves the boundary is live and versions agree. */
  version: (): Promise<CommandResult<'core_version'>> => call('core_version', {}),
  /** Host facts the UI adapts to. Never used to branch a security decision. */
  runtimeInfo: (): Promise<CommandResult<'runtime_info'>> => call('runtime_info', {}),
  /** Newest core log entries first. Omit the limit to read the whole buffer. */
  logs: (limit?: number): Promise<CommandResult<'app_logs'>> =>
    call('app_logs', limit === undefined ? {} : { limit }),
  /** Drops the in-memory log and returns how many entries were removed. */
  clearLogs: (): Promise<CommandResult<'clear_app_logs'>> => call('clear_app_logs', {}),

  settings: (): Promise<CommandResult<'get_settings'>> => call('get_settings', {}),
  updateSettings: (update: SettingsUpdate): Promise<CommandResult<'update_settings'>> =>
    call('update_settings', { update }),
  probeMirrors: (): Promise<CommandResult<'probe_mirrors'>> => call('probe_mirrors', {}),

  versions: (): Promise<CommandResult<'list_versions'>> => call('list_versions', {}),
  loaders: (loader: LoaderKind, gameVersion: string): Promise<CommandResult<'list_loaders'>> =>
    call('list_loaders', { loader, gameVersion }),

  java: (): Promise<CommandResult<'list_java'>> => call('list_java', {}),
  installJava: (major: number): Promise<CommandResult<'install_java'>> =>
    call('install_java', { major }),

  instances: (): Promise<CommandResult<'list_instances'>> => call('list_instances', {}),
  instance: (id: string): Promise<CommandResult<'get_instance'>> => call('get_instance', { id }),
  createInstance: (spec: InstanceSpec): Promise<CommandResult<'create_instance'>> =>
    call('create_instance', { spec }),
  updateInstance: (id: string, update: UpdateInstance): Promise<CommandResult<'update_instance'>> =>
    call('update_instance', { id, update }),
  deleteInstance: (id: string): Promise<CommandResult<'delete_instance'>> =>
    call('delete_instance', { id }),
  installInstance: (id: string): Promise<CommandResult<'install_instance'>> =>
    call('install_instance', { id }),

  launch: (id: string, options?: LaunchOptions): Promise<CommandResult<'launch_instance'>> =>
    call('launch_instance', options === undefined ? { id } : { id, options }),
  stop: (sessionId: string): Promise<CommandResult<'stop_instance'>> =>
    call('stop_instance', { sessionId }),
  sessions: (): Promise<CommandResult<'launch_sessions'>> => call('launch_sessions', {}),
  launchLogs: (sessionId: string, limit?: number): Promise<CommandResult<'launch_logs'>> =>
    call('launch_logs', limit === undefined ? { sessionId } : { sessionId, limit }),

  accounts: (): Promise<CommandResult<'list_accounts'>> => call('list_accounts', {}),
  addOffline: (username: string): Promise<CommandResult<'add_offline_account'>> =>
    call('add_offline_account', { args: { username } }),
  addYggdrasil: (
    authUrl: string,
    username: string,
    password: string,
  ): Promise<CommandResult<'add_yggdrasil_account'>> =>
    call('add_yggdrasil_account', { args: { authUrl, username, password } }),
  beginMicrosoft: (): Promise<CommandResult<'begin_microsoft_login'>> =>
    call('begin_microsoft_login', {}),
  completeMicrosoft: (
    deviceCode: string,
    interval?: number,
  ): Promise<CommandResult<'complete_microsoft_login'>> =>
    call(
      'complete_microsoft_login',
      interval === undefined ? { deviceCode } : { deviceCode, interval },
    ),
  selectAccount: (id: string): Promise<CommandResult<'select_account'>> =>
    call('select_account', { id }),
  removeAccount: (id: string): Promise<CommandResult<'remove_account'>> =>
    call('remove_account', { id }),
  refreshAccount: (id: string): Promise<CommandResult<'refresh_account'>> =>
    call('refresh_account', { id }),

  contentProviders: (): Promise<CommandResult<'content_providers'>> =>
    call('content_providers', {}),
  searchContent: (args: SearchContentArgs): Promise<CommandResult<'search_content'>> =>
    call('search_content', { args }),
  contentVersions: (args: ContentVersionsArgs): Promise<CommandResult<'content_versions'>> =>
    call('content_versions', { args }),
  installedContent: (instanceId: string): Promise<CommandResult<'installed_content'>> =>
    call('installed_content', { instanceId }),
  installContent: (args: InstallContentArgs): Promise<CommandResult<'install_content'>> =>
    call('install_content', { args }),
  removeContent: (
    instanceId: string,
    projectId: string,
  ): Promise<CommandResult<'remove_content'>> => call('remove_content', { instanceId, projectId }),
  exportInstance: (instanceId: string): Promise<CommandResult<'export_instance'>> =>
    call('export_instance', { instanceId }),
  importInstance: (json: string): Promise<CommandResult<'import_instance'>> =>
    call('import_instance', { args: { json } }),

  tasks: (): Promise<CommandResult<'list_tasks'>> => call('list_tasks', {}),
  cancelTask: (id: string): Promise<CommandResult<'cancel_task'>> => call('cancel_task', { id }),
} as const;

export type MnelysCore = typeof core;

/** Re-exported so callers can type a provider without importing the contract module. */
export type { ContentKind, ContentProviderId };
