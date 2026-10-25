import { create } from 'zustand';

/**
 * Ephemeral UI state only. ADR 0006 forbids duplicating core-owned data here: if the
 * Rust core is the source of truth for a value, that value lives in TanStack Query.
 */
export interface UiState {
  readonly taskCenterOpen: boolean;
  openTaskCenter: () => void;
  closeTaskCenter: () => void;
}

export const useUiStore = create<UiState>()((set) => ({
  taskCenterOpen: false,
  openTaskCenter: () => set({ taskCenterOpen: true }),
  closeTaskCenter: () => set({ taskCenterOpen: false }),
}));
