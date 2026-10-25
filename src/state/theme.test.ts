import { afterEach, describe, expect, it } from 'vitest';

import { applyTheme } from './theme';

afterEach(() => {
  delete document.documentElement.dataset['theme'];
});

describe('applyTheme', () => {
  it('lets the system preference win by removing the override', () => {
    document.documentElement.dataset['theme'] = 'dark';

    applyTheme('system');

    expect(document.documentElement).not.toHaveAttribute('data-theme');
  });

  it('writes an explicit palette to the root element', () => {
    applyTheme('light');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');

    applyTheme('dark');
    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
  });
});
