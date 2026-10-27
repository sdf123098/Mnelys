import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import type {
  ContentKind,
  ContentProviderId,
  ContentVersionsArgs,
  InstanceSpec,
  LaunchOptions,
  LoaderKind,
  SearchContentArgs,
  SettingsUpdate,
  UpdateInstance,
} from './commands';
import { core } from './client';

/** How many entries one panel refresh reads. The core buffer is bounded separately. */
const LOG_PAGE_SIZE = 200;

/** How often an open log panel asks the core for new entries. */
const LOG_POLL_INTERVAL_MS = 3000;

/** Instances change on disk as installs finish, so the list re-reads itself. */
const INSTANCE_POLL_INTERVAL_MS = 5000;

/** Running games report their state through a snapshot, not an event. */
const SESSION_POLL_INTERVAL_MS = 3000;

/** Background tasks are cheap to read and drive every progress bar. */
const TASK_POLL_INTERVAL_MS = 1000;

/**
 * Query keys for everything the Rust core owns. ADR 0006: keys are declared in one
 * module so that invalidation stays greppable and no component invents its own.
 */
export const queryKeys = {
  coreVersion: ['core', 'version'] as const,
  runtimeInfo: ['core', 'runtimeInfo'] as const,
  appLogs: ['core', 'appLogs'] as const,
  settings: ['core', 'settings'] as const,
  versions: ['core', 'versions'] as const,
  loaders: (loader: LoaderKind, gameVersion: string) =>
    ['core', 'loaders', loader, gameVersion] as const,
  java: ['core', 'java'] as const,
  instances: ['core', 'instances'] as const,
  instance: (id: string) => ['core', 'instance', id] as const,
  sessions: ['core', 'sessions'] as const,
  launchLogs: (sessionId: string) => ['core', 'launchLogs', sessionId] as const,
  accounts: ['core', 'accounts'] as const,
  providers: ['core', 'providers'] as const,
  search: (args: SearchContentArgs) => ['core', 'search', args] as const,
  contentVersions: (args: ContentVersionsArgs) => ['core', 'contentVersions', args] as const,
  installed: (instanceId: string) => ['core', 'installed', instanceId] as const,
  instanceFiles: (instanceId: string, dir: string) =>
    ['core', 'instanceFiles', instanceId, dir] as const,
  instanceLogs: (instanceId: string) => ['core', 'instanceLogs', instanceId] as const,
  instanceLogTail: (instanceId: string, path: string) =>
    ['core', 'instanceLogTail', instanceId, path] as const,
  instanceWorlds: (instanceId: string) => ['core', 'instanceWorlds', instanceId] as const,
  instanceSchematics: (instanceId: string) => ['core', 'instanceSchematics', instanceId] as const,
  diagnosis: (instanceId: string) => ['core', 'diagnosis', instanceId] as const,
  /** Scopes for invalidation: every listing of one instance, whatever its folder. */
  instanceFilesScope: (instanceId: string) => ['core', 'instanceFiles', instanceId] as const,
  instanceLogTailScope: (instanceId: string) => ['core', 'instanceLogTail', instanceId] as const,
  tasks: ['core', 'tasks'] as const,
};

/** The start-up handshake result. Cannot change while the app is running. */
export function useCoreVersion() {
  return useQuery({
    queryKey: queryKeys.coreVersion,
    queryFn: () => core.version(),
    staleTime: Number.POSITIVE_INFINITY,
    retry: 1,
  });
}

/** Host facts reported by the core. */
export function useRuntimeInfo() {
  return useQuery({
    queryKey: queryKeys.runtimeInfo,
    queryFn: () => core.runtimeInfo(),
    staleTime: Number.POSITIVE_INFINITY,
    retry: 1,
  });
}

/**
 * The core's in-memory log.
 *
 * Polling is opt-in: the panel passes `enabled` only while it is visible, so a closed
 * drawer does not keep the core awake.
 */
export function useAppLogs(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.appLogs,
    queryFn: () => core.logs(LOG_PAGE_SIZE),
    enabled,
    refetchInterval: enabled ? LOG_POLL_INTERVAL_MS : false,
    retry: 1,
  });
}

/** Clears the core log and refreshes every panel reading it. */
export function useClearAppLogs() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => core.clearLogs(),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.appLogs }),
  });
}

export function useSettings() {
  return useQuery({
    queryKey: queryKeys.settings,
    queryFn: () => core.settings(),
    retry: 1,
  });
}

export function useUpdateSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (update: SettingsUpdate) => core.updateSettings(update),
    onSuccess: (settings) => {
      queryClient.setQueryData(queryKeys.settings, settings);
    },
  });
}

export function useProbeMirrors() {
  return useMutation({ mutationFn: () => core.probeMirrors() });
}

/** Mojang's version manifest for the version picker. */
export function useVersions(enabled = true) {
  return useQuery({
    queryKey: queryKeys.versions,
    queryFn: () => core.versions(),
    enabled,
    staleTime: 10 * 60 * 1000,
    retry: 1,
  });
}

export function useLoaderVersions(loader: LoaderKind, gameVersion: string) {
  return useQuery({
    queryKey: queryKeys.loaders(loader, gameVersion),
    queryFn: () => core.loaders(loader, gameVersion),
    enabled: loader !== 'vanilla' && gameVersion.length > 0,
    staleTime: 5 * 60 * 1000,
    retry: 1,
  });
}

export function useJavaRuntimes() {
  return useQuery({
    queryKey: queryKeys.java,
    queryFn: () => core.java(),
    retry: 1,
  });
}

export function useInstallJava() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (major: number) => core.installJava(major),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.tasks }),
  });
}

export function useInstances() {
  return useQuery({
    queryKey: queryKeys.instances,
    queryFn: () => core.instances(),
    refetchInterval: INSTANCE_POLL_INTERVAL_MS,
    retry: 1,
  });
}

export function useInstance(id: string | undefined) {
  return useQuery({
    queryKey: queryKeys.instance(id ?? ''),
    queryFn: () => core.instance(id ?? ''),
    enabled: id !== undefined && id.length > 0,
    retry: 1,
  });
}

export function useCreateInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (spec: InstanceSpec) => core.createInstance(spec),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.instances }),
  });
}

export function useUpdateInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, update }: { id: string; update: UpdateInstance }) =>
      core.updateInstance(id, update),
    onSuccess: (summary) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.instances });
      queryClient.setQueryData(queryKeys.instance(summary.id), summary);
    },
  });
}

export function useDeleteInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => core.deleteInstance(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.instances }),
  });
}

export function useInstallInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => core.installInstance(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.tasks }),
  });
}

export function useLaunchInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, options }: { id: string; options?: LaunchOptions }) =>
      core.launch(id, options),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.sessions });
      void queryClient.invalidateQueries({ queryKey: queryKeys.instances });
    },
  });
}

export function useStopInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sessionId: string) => core.stop(sessionId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.sessions }),
  });
}

export function useLaunchSessions() {
  return useQuery({
    queryKey: queryKeys.sessions,
    queryFn: () => core.sessions(),
    refetchInterval: SESSION_POLL_INTERVAL_MS,
    retry: 1,
  });
}

export function useLaunchLogs(sessionId: string | undefined, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.launchLogs(sessionId ?? ''),
    queryFn: () => core.launchLogs(sessionId ?? '', 400),
    enabled: enabled && sessionId !== undefined && sessionId.length > 0,
    refetchInterval: enabled ? LOG_POLL_INTERVAL_MS : false,
    retry: 1,
  });
}

export function useAccounts() {
  return useQuery({
    queryKey: queryKeys.accounts,
    queryFn: () => core.accounts(),
    retry: 1,
  });
}

export function useAddOfflineAccount() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (username: string) => core.addOffline(username),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.accounts }),
  });
}

export function useAddYggdrasilAccount() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (values: { authUrl: string; username: string; password: string }) =>
      core.addYggdrasil(values.authUrl, values.username, values.password),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.accounts }),
  });
}

export function useBeginMicrosoftLogin() {
  return useMutation({ mutationFn: () => core.beginMicrosoft() });
}

export function useCompleteMicrosoftLogin() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (values: { deviceCode: string; interval?: number }) =>
      core.completeMicrosoft(values.deviceCode, values.interval),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.accounts }),
  });
}

export function useSelectAccount() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => core.selectAccount(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.accounts }),
  });
}

export function useRemoveAccount() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => core.removeAccount(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.accounts }),
  });
}

export function useRefreshAccount() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => core.refreshAccount(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.accounts }),
  });
}

export function useContentProviders() {
  return useQuery({
    queryKey: queryKeys.providers,
    queryFn: () => core.contentProviders(),
    staleTime: Number.POSITIVE_INFINITY,
    retry: 1,
  });
}

export function useSearchContent(args: SearchContentArgs | undefined) {
  return useQuery({
    queryKey: queryKeys.search(args ?? { provider: 'modrinth', query: '', kind: 'mod' }),
    queryFn: () => core.searchContent(args ?? { provider: 'modrinth', query: '', kind: 'mod' }),
    enabled: args !== undefined && args.query.trim().length > 0,
    staleTime: 60 * 1000,
    retry: 1,
  });
}

export function useContentVersions(args: ContentVersionsArgs | undefined) {
  return useQuery({
    queryKey: queryKeys.contentVersions(args ?? { provider: 'modrinth', projectId: '' }),
    queryFn: () => core.contentVersions(args ?? { provider: 'modrinth', projectId: '' }),
    enabled: args !== undefined && args.projectId.length > 0,
    staleTime: 60 * 1000,
    retry: 1,
  });
}

export function useInstalledContent(instanceId: string | undefined) {
  return useQuery({
    queryKey: queryKeys.installed(instanceId ?? ''),
    queryFn: () => core.installedContent(instanceId ?? ''),
    enabled: instanceId !== undefined && instanceId.length > 0,
    refetchInterval: INSTANCE_POLL_INTERVAL_MS,
    retry: 1,
  });
}

export function useInstallContent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: {
      instanceId: string;
      provider: ContentProviderId;
      projectId: string;
      versionId?: string;
      kind: ContentKind;
    }) => core.installContent(args),
    onSuccess: (_task, args) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.tasks });
      void queryClient.invalidateQueries({ queryKey: queryKeys.installed(args.instanceId) });
    },
  });
}

export function useRemoveContent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (values: { instanceId: string; projectId: string }) =>
      core.removeContent(values.instanceId, values.projectId),
    onSuccess: (_removed, values) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.installed(values.instanceId) });
    },
  });
}

export function useExportInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (instanceId: string) => core.exportInstance(instanceId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.tasks }),
  });
}

export function useImportInstance() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (json: string) => core.importInstance(json),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.instances }),
  });
}

/**
 * One directory inside an instance. `dir` is relative to the instance game directory
 * and only read while the panel is open, because a directory listing is pure disk I/O.
 */
export function useInstanceFiles(instanceId: string | undefined, dir: string, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.instanceFiles(instanceId ?? '', dir),
    queryFn: () => core.instanceFiles(instanceId ?? '', dir),
    enabled: enabled && instanceId !== undefined && instanceId.length > 0,
    staleTime: INSTANCE_POLL_INTERVAL_MS,
    retry: 1,
  });
}

/** The launcher logs and crash reports an instance has written. */
export function useInstanceLogs(instanceId: string | undefined, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.instanceLogs(instanceId ?? ''),
    queryFn: () => core.instanceLogs(instanceId ?? ''),
    enabled: enabled && instanceId !== undefined && instanceId.length > 0,
    refetchInterval: enabled ? LOG_POLL_INTERVAL_MS : false,
    retry: 1,
  });
}

/** The end of one log file. Pass `undefined` to close the reader. */
export function useInstanceLogTail(instanceId: string | undefined, path: string | undefined) {
  return useQuery({
    queryKey: queryKeys.instanceLogTail(instanceId ?? '', path ?? ''),
    queryFn: () => core.instanceLogTail(instanceId ?? '', path ?? ''),
    enabled:
      instanceId !== undefined && instanceId.length > 0 && path !== undefined && path.length > 0,
    staleTime: LOG_POLL_INTERVAL_MS,
    retry: 1,
  });
}

/** The saved worlds of one instance, with the size the core could measure. */
export function useInstanceWorlds(instanceId: string | undefined, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.instanceWorlds(instanceId ?? ''),
    queryFn: () => core.instanceWorlds(instanceId ?? ''),
    enabled: enabled && instanceId !== undefined && instanceId.length > 0,
    staleTime: INSTANCE_POLL_INTERVAL_MS,
    retry: 1,
  });
}

/** Schematic files found below `schematics/`. */
export function useInstanceSchematics(instanceId: string | undefined, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.instanceSchematics(instanceId ?? ''),
    queryFn: () => core.instanceSchematics(instanceId ?? ''),
    enabled: enabled && instanceId !== undefined && instanceId.length > 0,
    staleTime: INSTANCE_POLL_INTERVAL_MS,
    retry: 1,
  });
}

/**
 * Deletes one entry inside an instance. Every listing that could contain it is
 * re-read, because the core walks the directory rather than tracking entries.
 */
export function useDeleteInstanceEntry() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (values: { instanceId: string; path: string }) =>
      core.deleteInstanceEntry(values.instanceId, values.path),
    onSuccess: (_result, values) => {
      void queryClient.invalidateQueries({
        queryKey: queryKeys.instanceFilesScope(values.instanceId),
      });
      void queryClient.invalidateQueries({
        queryKey: queryKeys.instanceLogTailScope(values.instanceId),
      });
      void queryClient.invalidateQueries({ queryKey: queryKeys.instanceLogs(values.instanceId) });
      void queryClient.invalidateQueries({ queryKey: queryKeys.instanceWorlds(values.instanceId) });
      void queryClient.invalidateQueries({
        queryKey: queryKeys.instanceSchematics(values.instanceId),
      });
      void queryClient.invalidateQueries({ queryKey: queryKeys.installed(values.instanceId) });
    },
  });
}

/** Asks the desktop file manager to show one entry. */
export function useRevealInstanceEntry() {
  return useMutation({
    mutationFn: (values: { instanceId: string; path: string }) =>
      core.revealInstanceEntry(values.instanceId, values.path),
  });
}

/**
 * The rules run over the last launch of one instance. The core recomputes them from the
 * session buffer and the newest crash report, so the panel asks rather than polls.
 */
export function useInstanceDiagnosis(instanceId: string | undefined, enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.diagnosis(instanceId ?? ''),
    queryFn: () => core.diagnoseInstance(instanceId ?? ''),
    enabled: enabled && instanceId !== undefined && instanceId.length > 0,
    staleTime: LOG_POLL_INTERVAL_MS,
    retry: 1,
  });
}

/** Writes a redacted diagnosis bundle and reports where it went. */
export function useExportInstanceDiagnostics() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (instanceId: string) => core.exportInstanceDiagnostics(instanceId),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.appLogs }),
  });
}

/** Every background task, or nothing when the caller does not need it. */
export function useTasks(enabled: boolean) {
  return useQuery({
    queryKey: queryKeys.tasks,
    queryFn: () => core.tasks(),
    enabled,
    refetchInterval: (query) => {
      const tasks = query.state.data;
      const busy = tasks?.some((task) => task.state === 'queued' || task.state === 'running');
      return busy === true ? TASK_POLL_INTERVAL_MS : false;
    },
    retry: 1,
  });
}

export function useCancelTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: string) => core.cancelTask(id),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: queryKeys.tasks }),
  });
}
