import en from './locales/en.json';

/**
 * The catalog every other catalog is validated against. ADR 0006 types message keys
 * from `en`, so a key that exists only in a translation is a type error rather than a
 * string that silently renders nowhere.
 */
export const referenceCatalog = en;

type Catalog = typeof en;

/*
 * Catalogs are discovered from the folder rather than listed by hand. Adding a language
 * is therefore one JSON file, and the parity test picks it up automatically. They are
 * bundled eagerly: a desktop app has no network waterfall to avoid, and every locale is
 * available the moment the language picker opens.
 */
const catalogModules = import.meta.glob<{ default: Catalog }>('./locales/*.json', {
  eager: true,
});

const CATALOG_PREFIX = './locales/';
const CATALOG_SUFFIX = '.json';

function catalogCode(path: string): string {
  return path.slice(CATALOG_PREFIX.length, path.length - CATALOG_SUFFIX.length);
}

const catalogs = new Map<string, Catalog>(
  Object.entries(catalogModules).map(([path, module]) => [catalogCode(path), module.default]),
);

/** Picker order: the maintained core first, then every community catalog by code. */
const CORE_LANGUAGES = ['zh-Hans', 'zh-Hant', 'en', 'ja'];

export const SUPPORTED_LANGUAGES: readonly string[] = [
  ...CORE_LANGUAGES.filter((code) => catalogs.has(code)),
  ...[...catalogs.keys()].filter((code) => !CORE_LANGUAGES.includes(code)).sort(),
];

/** A BCP 47 tag with a bundled catalog. Resolved at runtime, not a closed union. */
export type SupportedLanguage = string;

/** Endonyms, so a picker entry is readable to the person looking for it. */
const LANGUAGE_NAMES: Readonly<Record<string, string>> = {
  en: 'English',
  'zh-Hans': '简体中文',
  'zh-Hant': '繁體中文',
  ja: '日本語',
  ko: '한국어',
  de: 'Deutsch',
  fr: 'Français',
  es: 'Español',
  'pt-BR': 'Português (Brasil)',
  it: 'Italiano',
  pl: 'Polski',
  nl: 'Nederlands',
  sv: 'Svenska',
  da: 'Dansk',
  nb: 'Norsk bokmål',
  fi: 'Suomi',
  cs: 'Čeština',
  sk: 'Slovenčina',
  hu: 'Magyar',
  ro: 'Română',
  bg: 'Български',
  el: 'Ελληνικά',
  ru: 'Русский',
  uk: 'Українська',
  tr: 'Türkçe',
  vi: 'Tiếng Việt',
  id: 'Bahasa Indonesia',
  ms: 'Bahasa Melayu',
  th: 'ไทย',
  hi: 'हिन्दी',
  bn: 'বাংলা',
  ta: 'தமிழ்',
  ar: 'العربية',
  he: 'עברית',
  fa: 'فارسی',
  ur: 'اردو',
};

export const LANGUAGE_LABELS: Readonly<Record<SupportedLanguage, string>> = Object.fromEntries(
  SUPPORTED_LANGUAGES.map((code) => [code, LANGUAGE_NAMES[code] ?? code]),
);

const RTL_LANGUAGES = new Set(['ar', 'he', 'fa', 'ur']);

export function isRtlLanguage(language: SupportedLanguage): boolean {
  return RTL_LANGUAGES.has(language.split('-')[0] ?? '');
}

function normalize(tag: string): string {
  return tag.trim().toLowerCase().replace(/_/g, '-');
}

/**
 * Maps an arbitrary browser tag onto a bundled catalog. Exact region wins, then the
 * same base language, then the Chinese script fallback, then English.
 */
export function matchLanguage(tag: string): SupportedLanguage {
  const value = normalize(tag);
  if (value.length === 0) {
    return 'en';
  }

  const exact = SUPPORTED_LANGUAGES.find((code) => normalize(code) === value);
  if (exact !== undefined) {
    return exact;
  }

  if (value.startsWith('zh')) {
    const traditional = /(?:^|-)hant(?:-|$)|-(?:tw|hk|mo)(?:-|$)/.test(value);
    const preferred = traditional ? 'zh-Hant' : 'zh-Hans';
    return catalogs.has(preferred) ? preferred : 'zh-Hans';
  }

  const base = value.split('-')[0] ?? '';
  const sameBase = SUPPORTED_LANGUAGES.find((code) => normalize(code).split('-')[0] === base);
  return sameBase ?? 'en';
}

export const resources: Record<string, { translation: Catalog }> = Object.fromEntries(
  [...catalogs].map(([code, catalog]) => [code, { translation: catalog }]),
);
