import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import { resources, SUPPORTED_LANGUAGES, type SupportedLanguage } from './resources';

export const LANGUAGE_STORAGE_KEY = 'mnelys.language';

export function isSupportedLanguage(value: unknown): value is SupportedLanguage {
  return typeof value === 'string' && (SUPPORTED_LANGUAGES as readonly string[]).includes(value);
}

/**
 * Resolution order: a previously stored choice, then the browser language, then English.
 * The stored choice reads from `localStorage`, which the webview persists per install.
 */
export function resolveInitialLanguage(stored?: string | null): SupportedLanguage {
  let candidate = stored;
  if (candidate === undefined) {
    try {
      candidate = globalThis.localStorage?.getItem(LANGUAGE_STORAGE_KEY) ?? null;
    } catch {
      // A blocked localStorage must never stop the app from starting.
      candidate = null;
    }
  }
  if (isSupportedLanguage(candidate)) {
    return candidate;
  }
  const preferred = typeof navigator === 'undefined' ? '' : navigator.language;
  if (preferred.startsWith('zh')) return 'zh-Hans';
  if (preferred.startsWith('ja')) return 'ja';
  return 'en';
}

export function persistLanguage(language: SupportedLanguage): void {
  try {
    globalThis.localStorage?.setItem(LANGUAGE_STORAGE_KEY, language);
  } catch {
    // Persisting the preference is best-effort.
  }
}

void i18n.use(initReactI18next).init({
  resources,
  lng: resolveInitialLanguage(),
  fallbackLng: 'en',
  supportedLngs: [...SUPPORTED_LANGUAGES],
  interpolation: { escapeValue: false },
  returnNull: false,
});

export default i18n;
