import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/**
 * Shell preferences that have no core-side owner yet. They are presentation choices,
 * not product data: when settings move into the Rust core these keys become a one-way
 * import rather than a second source of truth.
 */
export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;

export type ThemePreference = (typeof THEME_PREFERENCES)[number];

export interface PreferencesState {
  readonly theme: ThemePreference;
  readonly developerMode: boolean;
  setTheme: (theme: ThemePreference) => void;
  setDeveloperMode: (enabled: boolean) => void;
}

export const PREFERENCES_STORAGE_KEY = 'mnelys.preferences';

export const usePreferences = create<PreferencesState>()(
  persist(
    (set) => ({
      theme: 'system',
      developerMode: false,
      setTheme: (theme) => set({ theme }),
      setDeveloperMode: (developerMode) => set({ developerMode }),
    }),
    {
      name: PREFERENCES_STORAGE_KEY,
      version: 1,
    },
  ),
);
