/**
 * Type-safe message keys (ADR 0006).
 *
 * `CustomTypeOptions.resources` is pointed at the English catalog, so `t('nav.instances')`
 * is checked at compile time and a missing key is a build failure rather than a raw key
 * rendered at runtime.
 */
import type referenceCatalog from './locales/en.json';

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'translation';
    resources: {
      translation: typeof referenceCatalog;
    };
  }
}
