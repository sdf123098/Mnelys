import { describe, expect, it } from 'vitest';

import en from './locales/en.json';
import ja from './locales/ja.json';
import zhHans from './locales/zh-Hans.json';

type CatalogNode = string | { readonly [key: string]: CatalogNode };

function flatten(node: CatalogNode, prefix = ''): string[] {
  if (typeof node === 'string') {
    return [prefix];
  }
  return Object.entries(node).flatMap(([key, value]) =>
    flatten(value, prefix.length > 0 ? `${prefix}.${key}` : key),
  );
}

/**
 * ADR 0006 types message keys from the English catalog. That catches a key the code
 * invents, but not a key a translation is missing — so parity is asserted here.
 */
describe('message catalogs', () => {
  const reference = flatten(en).sort();

  it('has a non-trivial reference catalog', () => {
    expect(reference.length).toBeGreaterThan(10);
  });

  it('Simplified Chinese declares exactly the reference keys', () => {
    expect(flatten(zhHans).sort()).toEqual(reference);
  });

  it('Japanese declares exactly the reference keys', () => {
    expect(flatten(ja).sort()).toEqual(reference);
  });
});
