import { describe, expect, it } from 'vitest';

import { DIAGNOSIS_IDS } from '@/ipc/commands';

import en from './locales/en.json';

type CatalogNode = string | { readonly [key: string]: CatalogNode };

function flatten(node: CatalogNode, prefix = ''): string[] {
  if (typeof node === 'string') {
    return [prefix];
  }
  return Object.entries(node).flatMap(([key, value]) =>
    flatten(value, prefix.length > 0 ? `${prefix}.${key}` : key),
  );
}

const modules = import.meta.glob<{ default: CatalogNode }>('./locales/*.json', { eager: true });

const catalogs = Object.entries(modules)
  .map(([path, module]) => ({
    code: path.replace('./locales/', '').replace('.json', ''),
    catalog: module.default,
  }))
  .filter(({ code }) => code !== 'en');

/**
 * ADR 0006 types message keys from the English catalog. That catches a key the code
 * invents, but not a key a translation is missing — so parity is asserted here for
 * every bundled catalog, including ones added later by dropping in a JSON file.
 */
describe('message catalogs', () => {
  const reference = flatten(en).sort();

  it('has a non-trivial reference catalog', () => {
    expect(reference.length).toBeGreaterThan(10);
  });

  it('bundles at least the three maintained languages', () => {
    expect(catalogs.map(({ code }) => code)).toEqual(expect.arrayContaining(['ja', 'zh-Hans']));
  });

  it.each(catalogs)('$code declares exactly the reference keys', ({ catalog }) => {
    expect(flatten(catalog).sort()).toEqual(reference);
  });

  it('product names are single-language, not bilingual', () => {
    expect(en.app.name).toBe('Mnelys');
    for (const { code, catalog } of catalogs) {
      const name = (catalog as { app?: { name?: string } }).app?.name;
      expect(name, `${code} must name the product`).toBeTruthy();
      expect(name, `${code} must not append a second name`).not.toContain('/');
    }
  });

  /**
   * The core ships rule ids, not sentences, so a rule with no catalog entry would render
   * its raw key. Both directions are checked: a rule the interface forgot to translate and
   * a catalog entry for a rule the core no longer reports.
   */
  it('carries one title and one fix per diagnosis rule', () => {
    expect(Object.keys(en.diagnosis.rules).sort()).toEqual([...DIAGNOSIS_IDS].sort());
    for (const id of DIAGNOSIS_IDS) {
      expect(Object.keys(en.diagnosis.rules[id]).sort()).toEqual(['fix', 'title']);
    }
  });
});
