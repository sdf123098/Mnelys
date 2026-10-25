import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';

import { core } from './client';

/** How many entries one panel refresh reads. The core buffer is bounded separately. */
const LOG_PAGE_SIZE = 200;

/** How often an open log panel asks the core for new entries. */
const LOG_POLL_INTERVAL_MS = 3000;

/**
 * Query keys for everything the Rust core owns. ADR 0006: keys are declared in one
 * module so that invalidation stays greppable and no component invents its own.
 */
export const queryKeys = {
  coreVersion: ['core', 'version'] as const,
  runtimeInfo: ['core', 'runtimeInfo'] as const,
  appLogs: ['core', 'appLogs'] as const,
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
