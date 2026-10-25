import { create } from 'zustand';
import { persist } from 'zustand/middleware';

/**
 * Shell preferences that have no core-side owner yet. They are presentation choices,
 * not product data: when settings move into the Rust core these keys become a one-way
 * import rather than a second source of truth.
 */
export const THEME_PREFERENCES = ['system', 'light', 'dark'] as const;

export type ThemePreference = (typeof THEME_PREFERENCES)[number];

/** The brand accent. Stored as the sentinel so a palette change reaches every profile. */
export const BRAND_ACCENT = 'brand';

export interface PreferencesState {
  readonly theme: ThemePreference;
  /** Either {@link BRAND_ACCENT} or a `#RRGGBB` value. */
  readonly accent: string;
  readonly developerMode: boolean;
  setTheme: (theme: ThemePreference) => void;
  setAccent: (accent: string) => void;
  setDeveloperMode: (enabled: boolean) => void;
}

export const PREFERENCES_STORAGE_KEY = 'mnelys.preferences';

interface StoredPreferences {
  readonly theme?: ThemePreference;
  readonly accent?: string;
  readonly developerMode?: boolean;
}

export const usePreferences = create<PreferencesState>()(
  persist(
    (set) => ({
      theme: 'system',
      accent: BRAND_ACCENT,
      developerMode: false,
      setTheme: (theme) => set({ theme }),
      setAccent: (accent) => set({ accent }),
      setDeveloperMode: (developerMode) => set({ developerMode }),
    }),
    {
      name: PREFERENCES_STORAGE_KEY,
      version: 2,
      migrate: (persisted) => {
        // v1 had no accent; the brand default is the only sensible carry-over.
        const previous = (persisted ?? {}) as StoredPreferences;
        return {
          theme: previous.theme ?? 'system',
          accent: previous.accent ?? BRAND_ACCENT,
          developerMode: previous.developerMode ?? false,
        } satisfies StoredPreferences;
      },
    },
  ),
);
