import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Switch } from './Switch';

describe('Switch', () => {
  it('is announced as a switch and reports the next value', async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <Switch
        label="Developer mode"
        checked={false}
        onChange={onChange}
        description="Shows tools."
      />,
    );

    const control = screen.getByRole('switch', { name: 'Developer mode' });
    await user.click(control);

    expect(onChange).toHaveBeenCalledWith(true);
  });
});
