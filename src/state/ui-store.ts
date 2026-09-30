import { create } from 'zustand';

/**
 * Ephemeral UI state only. ADR 0006 forbids duplicating core-owned data here: if the
 * Rust core is the source of truth for a value, that value lives in TanStack Query.
 */

export interface UiState {
  /** Last section the user visited, so a restart can return there. */
  readonly lastNavSection: string;
  setLastNavSection: (section: string) => void;
  readonly logPanelOpen: boolean;
  toggleLogPanel: () => void;
}

export const useUiStore = create<UiState>()((set) => ({
  lastNavSection: 'instances',
  setLastNavSection: (section) => set({ lastNavSection: section }),
  logPanelOpen: false,
  toggleLogPanel: () => set((state) => ({ logPanelOpen: !state.logPanelOpen })),
}));
