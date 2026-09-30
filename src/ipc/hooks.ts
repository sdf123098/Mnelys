import { useQuery } from '@tanstack/react-query';

import { core } from './client';

/**
 * Query keys for everything the Rust core owns. ADR 0006: keys are declared in one
 * module so that invalidation stays greppable and no component invents its own.
 */
export const queryKeys = {
  coreVersion: ['core', 'version'] as const,
  runtimeInfo: ['core', 'runtimeInfo'] as const,
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
