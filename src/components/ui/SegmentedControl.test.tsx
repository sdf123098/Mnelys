import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { SegmentedControl, type SegmentOption } from './SegmentedControl';

type Theme = 'system' | 'light' | 'dark';

const OPTIONS: readonly SegmentOption<Theme>[] = [
  { value: 'system', label: 'System', icon: 'monitor' },
  { value: 'light', label: 'Light', icon: 'sun' },
  { value: 'dark', label: 'Dark', icon: 'moon' },
];

describe('SegmentedControl', () => {
  it('is a labelled native radio group and reports the chosen value', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <SegmentedControl
        name="theme"
        label="Appearance"
        value="system"
        options={OPTIONS}
        onChange={onChange}
      />,
    );

    expect(screen.getByRole('group', { name: 'Appearance' })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: 'System' })).toBeChecked();

    await user.click(screen.getByRole('radio', { name: 'Dark' }));

    expect(onChange).toHaveBeenCalledWith('dark');
  });
});
