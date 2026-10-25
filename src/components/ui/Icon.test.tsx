import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Icon } from './Icon';

describe('Icon', () => {
  it('renders a decorative svg by default', () => {
    const { container } = render(<Icon name="instances" />);
    const svg = container.querySelector('svg');

    expect(svg).not.toBeNull();
    expect(svg).toHaveAttribute('aria-hidden', 'true');
    expect(svg).toHaveAttribute('focusable', 'false');
  });

  it('exposes a name only when the icon carries meaning', () => {
    render(<Icon name="warning" label="Warning" />);

    expect(screen.getByRole('img', { name: 'Warning' })).toBeInTheDocument();
  });
});
