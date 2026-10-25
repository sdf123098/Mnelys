import { act, render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, beforeAll, beforeEach, describe, expect, it } from 'vitest';

import i18n from '@/i18n';
import { usePreferences } from '@/state/preferences-store';

import { NavSidebar } from './NavSidebar';

beforeAll(async () => {
  await i18n.changeLanguage('en');
});

beforeEach(() => {
  act(() => {
    usePreferences.setState({ developerMode: false });
  });
});

afterEach(() => {
  act(() => {
    usePreferences.setState({ theme: 'system', developerMode: false });
  });
});

function renderSidebar() {
  return render(
    <MemoryRouter>
      <NavSidebar />
    </MemoryRouter>,
  );
}

describe('NavSidebar', () => {
  it('renders the primary sections and hides developer tools by default', () => {
    renderSidebar();

    expect(screen.getByRole('link', { name: 'Start' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Instances' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Downloads' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Multiplayer' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: 'Developer' })).not.toBeInTheDocument();
    expect(screen.getByText('M1 shell')).toBeInTheDocument();
  });

  it('reveals the developer section when developer mode is on', () => {
    act(() => {
      usePreferences.setState({ developerMode: true });
    });

    renderSidebar();

    expect(screen.getByRole('link', { name: 'Developer' })).toBeInTheDocument();
  });
});
