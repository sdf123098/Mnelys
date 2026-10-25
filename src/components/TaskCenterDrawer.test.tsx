import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';

import i18n from '@/i18n';
import type { AppLogEntry } from '@/ipc/commands';
import { core } from '@/ipc/client';
import { useUiStore } from '@/state/ui-store';

import { TaskCenterDrawer } from './TaskCenterDrawer';

vi.mock('@/ipc/client', () => ({
  core: {
    version: vi.fn(),
    runtimeInfo: vi.fn(),
    logs: vi.fn(),
    clearLogs: vi.fn(),
  },
}));

const LOG_ENTRIES: readonly AppLogEntry[] = [
  {
    id: 2,
    timestampMs: Date.UTC(2026, 9, 26, 12, 0, 2),
    level: 'info',
    target: 'core',
    message: 'frontend handshake with core 0.1.0',
  },
  {
    id: 1,
    timestampMs: Date.UTC(2026, 9, 26, 12, 0, 1),
    level: 'debug',
    target: 'core.runtime',
    message: '3 commands registered',
  },
];

function renderDrawer() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <TaskCenterDrawer />
    </QueryClientProvider>,
  );
}

function openDrawer() {
  act(() => {
    useUiStore.setState({ taskCenterOpen: true });
  });
  return renderDrawer();
}

beforeAll(async () => {
  await i18n.changeLanguage('en');
});

beforeEach(() => {
  act(() => {
    useUiStore.setState({ taskCenterOpen: false });
  });
  vi.mocked(core.logs).mockResolvedValue([...LOG_ENTRIES]);
  vi.mocked(core.clearLogs).mockResolvedValue(LOG_ENTRIES.length);
});

afterEach(() => {
  act(() => {
    useUiStore.setState({ taskCenterOpen: false });
  });
  vi.clearAllMocks();
});

describe('TaskCenterDrawer', () => {
  it('renders as a modal dialog once opened', () => {
    openDrawer();

    const dialog = screen.getByRole('dialog', { name: 'Task center' });
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(screen.getByRole('button', { name: 'Close task center' })).toHaveFocus();
  });

  it('closes on Escape and returns to the closed state', async () => {
    const user = userEvent.setup();
    openDrawer();

    await user.keyboard('{Escape}');

    expect(useUiStore.getState().taskCenterOpen).toBe(false);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('renders nothing while closed', () => {
    const { container } = renderDrawer();
    expect(container).toBeEmptyDOMElement();
  });

  it('reads the core log only after the logs section is selected', async () => {
    const user = userEvent.setup();
    openDrawer();

    expect(core.logs).not.toHaveBeenCalled();
    await user.click(screen.getByRole('radio', { name: 'Logs' }));

    expect(await screen.findByText('frontend handshake with core 0.1.0')).toBeInTheDocument();
    expect(screen.getByText('Entries: 2')).toBeInTheDocument();
    expect(core.logs).toHaveBeenCalledWith(200);
  });

  it('clears the log only after the confirmation is accepted', async () => {
    const user = userEvent.setup();
    openDrawer();

    await user.click(screen.getByRole('radio', { name: 'Logs' }));
    await screen.findByText('frontend handshake with core 0.1.0');

    await user.click(screen.getByRole('button', { name: 'Clear logs' }));

    const confirmation = screen.getByRole('dialog', { name: 'Clear all log entries?' });
    expect(core.clearLogs).not.toHaveBeenCalled();

    await user.click(within(confirmation).getByRole('button', { name: 'Clear logs' }));

    await waitFor(() => {
      expect(core.clearLogs).toHaveBeenCalledTimes(1);
    });
    await waitFor(() => {
      expect(
        screen.queryByRole('dialog', { name: 'Clear all log entries?' }),
      ).not.toBeInTheDocument();
    });
  });
});
