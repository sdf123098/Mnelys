import en from './locales/en.json';
import ja from './locales/ja.json';
import zhHans from './locales/zh-Hans.json';

/**
 * The catalog every other catalog is validated against. ADR 0006 types message keys
 * from `en`, so a key that exists only in a translation is a type error rather than a
 * string that silently renders nowhere.
 */
export const referenceCatalog = en;

export const SUPPORTED_LANGUAGES = ['zh-Hans', 'en', 'ja'] as const;

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number];

export const LANGUAGE_LABELS: Readonly<Record<SupportedLanguage, string>> = {
  'zh-Hans': '简体中文',
  en: 'English',
  ja: '日本語',
};

export const resources = {
  'zh-Hans': { translation: zhHans },
  en: { translation: en },
  ja: { translation: ja },
} as const;
