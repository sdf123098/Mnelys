import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { PlaceholderPanel } from './PlaceholderPanel';

describe('PlaceholderPanel', () => {
  it('renders the heading and body it is handed', () => {
    render(<PlaceholderPanel heading="Instances" body="No instances yet." />);

    expect(screen.getByRole('heading', { level: 1, name: 'Instances' })).toBeInTheDocument();
    expect(screen.getByText('No instances yet.')).toBeInTheDocument();
  });

  it('renders children inside the panel', () => {
    render(
      <PlaceholderPanel heading="Settings" body="About">
        <p>extra content</p>
      </PlaceholderPanel>,
    );

    expect(screen.getByText('extra content')).toBeInTheDocument();
  });
});
