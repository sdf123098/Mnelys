import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { mockIPC } from '@tauri-apps/api/mocks';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { beforeAll, beforeEach, describe, expect, it } from 'vitest';

import i18n from '@/i18n';
import type {
  AccountSummary,
  ContentProject,
  ContentSearchPage,
  DiagnosisReport,
  DiagnosticsBundle,
  FileListing,
  InstalledContent,
  InstanceSummary,
  JavaRuntime,
  LaunchSession,
  LogListing,
  LogTail,
  SettingsView,
  TaskSnapshot,
  VersionManifest,
  WorkspaceEntry,
  WorldEntry,
} from '@/ipc/commands';
import { usePreferences } from '@/state/preferences-store';

import { ContentRoute } from './ContentRoute';
import { DeveloperRoute } from './DeveloperRoute';
import { DownloadsRoute } from './DownloadsRoute';
import { InstancesRoute } from './InstancesRoute';
import { SettingsRoute } from './SettingsRoute';

const INSTANCE: InstanceSummary = {
  id: 'demo',
  name: 'Demo pack',
  gameVersion: '1.21.4',
  loader: 'fabric',
  loaderVersion: '0.16.9',
  versionId: '1.21.4-fabric-0.16.9',
  createdAtMs: Date.UTC(2026, 9, 20),
  lastLaunchedMs: Date.UTC(2026, 9, 25),
  installed: true,
  directory: 'D:/Mnelys/instances/demo',
};

const SECOND: InstanceSummary = {
  id: 'vanilla',
  name: 'Plain 1.21.4',
  gameVersion: '1.21.4',
  loader: 'vanilla',
  versionId: '1.21.4',
  createdAtMs: Date.UTC(2026, 9, 21),
  installed: false,
  directory: 'D:/Mnelys/instances/vanilla',
};

const SESSION: LaunchSession = {
  id: 'session-1',
  instanceId: 'demo',
  versionId: INSTANCE.versionId,
  pid: 4242,
  state: 'running',
  startedAtMs: Date.UTC(2026, 9, 26),
};

const ACCOUNT: AccountSummary = {
  id: 'acc-1',
  kind: 'offline',
  name: 'Tester',
  uuid: '0facc2c4b0d34a0e9d1e1d0e2f3a4b5c',
  selected: true,
  addedAtMs: Date.UTC(2026, 9, 1),
};

const PROJECT: ContentProject = {
  id: 'sodium',
  provider: 'modrinth',
  kind: 'mod',
  slug: 'sodium',
  name: 'Sodium',
  summary: 'A modern rendering engine for Minecraft which greatly improves frame rates.',
  downloads: 32_145_678,
  author: 'jellysquid3',
};

const JAVA: JavaRuntime[] = [
  {
    path: 'D:/Mnelys/runtime/21/bin/java.exe',
    major: 21,
    version: '21.0.5+11',
    vendor: 'Eclipse Adoptium',
    source: 'managed',
  },
];

const TASK: TaskSnapshot = {
  id: 'task-1',
  kind: 'download',
  state: 'running',
  label: 'client.jar',
  progress: { current: 4_200_000, total: 12_000_000, message: 'sni/mojang' },
  startedAtMs: Date.UTC(2026, 9, 26),
};

const SETTINGS: SettingsView = {
  mirror: 'auto',
  activeMirror: 'direct',
  memoryMb: 4096,
  root: 'D:/Mnelys',
};

const MANIFEST: VersionManifest = {
  latest: { release: '1.21.4', snapshot: '25w01a' },
  versions: [
    {
      id: '1.21.4',
      type: 'release',
      url: 'https://piston-meta.mojang.com/1.21.4.json',
      releaseTime: '2024-12-03T10:00:00+00:00',
    },
    { id: '25w01a', type: 'snapshot', url: 'https://piston-meta.mojang.com/25w01a.json' },
  ],
};

const MODIFIED = Date.UTC(2026, 9, 25, 8, 30);

const INSTALLED: InstalledContent[] = [
  {
    provider: 'modrinth',
    projectId: 'sodium',
    versionId: 'version-1',
    name: 'Sodium',
    filename: 'sodium-fabric-0.6.0.jar',
    kind: 'mod',
    installedAtMs: MODIFIED,
  },
];

const WORLD: WorldEntry = {
  name: 'New World',
  path: 'saves/New World',
  sizeBytes: 1_073_741_824,
  sizeComplete: true,
  modifiedMs: MODIFIED,
};

const SCHEMATIC: WorkspaceEntry = {
  name: 'castle.schem',
  path: 'schematics/castle.schem',
  isDir: false,
  sizeBytes: 524_288,
  modifiedMs: MODIFIED,
};

const CRASH: WorkspaceEntry = {
  name: 'crash-2026.txt',
  path: 'crash-reports/crash-2026.txt',
  isDir: false,
  sizeBytes: 4_096,
  modifiedMs: MODIFIED,
};

const LOG: WorkspaceEntry = {
  name: 'latest.log',
  path: 'logs/latest.log',
  isDir: false,
  sizeBytes: 98_304,
  modifiedMs: MODIFIED,
};

const ROOT_FILE: WorkspaceEntry = {
  name: 'options.txt',
  path: 'options.txt',
  isDir: false,
  sizeBytes: 2_048,
  modifiedMs: MODIFIED,
};

/** What the core's rules report for the demo instance's last run. */
const DIAGNOSIS: DiagnosisReport = {
  instanceId: 'demo',
  generatedAtMs: Date.UTC(2026, 9, 26),
  session: SESSION,
  findings: [
    {
      id: 'out-of-memory',
      severity: 'critical',
      params: { subtype: 'java heap space' },
      evidence: [
        {
          source: 'launchLog',
          path: LOG.path,
          line: 128,
          excerpt: 'java.lang.OutOfMemoryError: Java heap space',
        },
      ],
    },
  ],
  analyzedLines: 4_096,
  logTruncated: false,
  crashReport: CRASH.path,
};

function handle(command: string): unknown {
  switch (command) {
    case 'list_instances':
      return [INSTANCE, SECOND];
    case 'get_instance':
      return INSTANCE;
    case 'launch_sessions':
      return [SESSION];
    case 'list_accounts':
      return [ACCOUNT];
    case 'list_java':
      return JAVA;
    case 'list_tasks':
      return [TASK];
    case 'get_settings':
      return SETTINGS;
    case 'list_versions':
      return MANIFEST;
    case 'content_providers':
      return ['modrinth'];
    case 'search_content':
      return {
        provider: 'modrinth',
        hits: [PROJECT],
        offset: 0,
        limit: 20,
        total: 1,
      } satisfies ContentSearchPage;
    case 'installed_content':
      return INSTALLED;
    case 'launch_logs':
      return ['[Server thread/INFO]: Preparing spawn area'];
    case 'list_instance_files':
      return {
        path: '',
        entries: [
          { name: 'mods', path: 'mods', isDir: true, sizeBytes: 0, modifiedMs: MODIFIED },
          ROOT_FILE,
        ],
        truncated: false,
      } satisfies FileListing;
    case 'list_instance_logs':
      return { logs: [LOG], crashReports: [CRASH] } satisfies LogListing;
    case 'read_instance_log':
      return {
        path: CRASH.path,
        text: 'java.lang.OutOfMemoryError: Java heap space',
        truncated: true,
      } satisfies LogTail;
    case 'list_instance_worlds':
      return [WORLD];
    case 'list_instance_schematics':
      return [SCHEMATIC];
    case 'diagnose_instance':
      return DIAGNOSIS;
    case 'export_instance_diagnostics':
      return {
        path: 'D:/Mnelys/data/diagnostics/demo-1.md',
        sizeBytes: 2_048,
        findings: 1,
      } satisfies DiagnosticsBundle;
    default:
      return null;
  }
}

function renderAt(path: string, element: React.ReactElement) {
  const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={client}>
      <MemoryRouter initialEntries={[path]}>
        <Routes>
          <Route path="/instances" element={element} />
          <Route path="/instances/:id" element={element} />
          <Route path="*" element={element} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

beforeAll(async () => {
  await i18n.changeLanguage('en');
});

beforeEach(() => {
  mockIPC(handle);
  usePreferences.setState({ developerMode: true });
});

describe('rewritten routes render real core data', () => {
  it('shows the instance workspace with one launch action', async () => {
    const user = userEvent.setup();
    renderAt('/instances/demo', <InstancesRoute />);
    expect(await screen.findByRole('heading', { name: 'Demo pack' })).toBeInTheDocument();
    expect(screen.getByText('Plain 1.21.4')).toBeInTheDocument();

    // The selected instance is running, so its single primary action is stop.
    expect(screen.getByRole('button', { name: 'Stop' })).toBeEnabled();
    expect(screen.getByText('Launching as Tester.')).toBeInTheDocument();
    expect(screen.queryByText('How instances will work')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Instance settings' }));
    const dialog = await screen.findByRole('dialog', { name: 'Instance settings' });
    expect(within(dialog).getByRole('radio', { name: 'Logs' })).toBeInTheDocument();
    expect(within(dialog).getByLabelText('Java path')).toHaveValue('');
    expect(
      within(dialog).queryByRole('button', { name: 'Browse the content hub' }),
    ).not.toBeInTheDocument();

    await user.click(within(dialog).getByRole('radio', { name: 'Content' }));
    expect(
      await within(dialog).findByRole('link', { name: 'Browse the content hub' }),
    ).toHaveAttribute('href', '/content?instance=demo');

    // The tabs browse one type at a time; a search term spans every type, locally.
    await user.click(within(dialog).getByRole('radio', { name: 'Resource packs' }));
    expect(within(dialog).queryByText('Sodium')).not.toBeInTheDocument();

    const search = within(dialog).getByLabelText('Search installed content');
    await user.type(search, 'sodium');
    expect(within(dialog).getByText('Sodium')).toBeInTheDocument();
    expect(
      within(dialog).getByText('Search covers every type, not only the selected tab.'),
    ).toBeInTheDocument();

    await user.clear(search);
    await user.type(search, 'no-such-mod');
    expect(within(dialog).getByText('Nothing installed matches this filter.')).toBeInTheDocument();
  });

  it('lists worlds, logs and files of the instance folder', async () => {
    const user = userEvent.setup();
    renderAt('/instances/demo', <InstancesRoute />);
    await user.click(await screen.findByRole('button', { name: 'Instance settings' }));
    const dialog = await screen.findByRole('dialog', { name: 'Instance settings' });

    await user.click(within(dialog).getByRole('radio', { name: 'Worlds' }));
    expect(await within(dialog).findByText('New World')).toBeInTheDocument();
    expect(within(dialog).getByText(/1.0 GiB/)).toBeInTheDocument();

    await user.click(within(dialog).getByRole('button', { name: 'Delete' }));
    expect(
      within(dialog).getByText(
        '“New World” and everything inside it are deleted. This cannot be undone.',
      ),
    ).toBeInTheDocument();

    await user.click(within(dialog).getByRole('radio', { name: 'Logs' }));
    const crashes = await within(dialog).findByRole('region', { name: 'Crash reports' });
    await user.click(within(crashes).getByRole('button', { name: 'Show end' }));
    expect(
      await within(dialog).findByText('java.lang.OutOfMemoryError: Java heap space'),
    ).toBeInTheDocument();
    expect(within(dialog).getByText('Only the end of this file is read.')).toBeInTheDocument();

    await user.click(within(dialog).getByRole('radio', { name: 'Files' }));
    expect(await within(dialog).findByText('options.txt')).toBeInTheDocument();
    expect(within(dialog).getByText('Instance folder')).toBeInTheDocument();

    await user.click(within(dialog).getByRole('radio', { name: 'Schematics' }));
    expect(await within(dialog).findByText('castle.schem')).toBeInTheDocument();
    expect(within(dialog).getByText(/512 KiB/)).toBeInTheDocument();
  });

  /**
   * The diagnosis pane is the core's rule output, not a sentence written in the shell:
   * the command ships an id and a value, and the catalog turns them into this text.
   */
  it('diagnoses the last run and exports a bundle', async () => {
    const user = userEvent.setup();
    renderAt('/instances/demo', <InstancesRoute />);
    await user.click(await screen.findByRole('button', { name: 'Instance settings' }));
    const dialog = await screen.findByRole('dialog', { name: 'Instance settings' });

    await user.click(within(dialog).getByRole('radio', { name: 'Diagnosis' }));
    expect(await within(dialog).findByText('The game ran out of memory')).toBeInTheDocument();
    expect(within(dialog).getByText('Blocking')).toBeInTheDocument();
    expect(within(dialog).getByText('java heap space')).toBeInTheDocument();
    expect(within(dialog).getByText(/latest\.log:128/)).toBeInTheDocument();
    expect(
      within(dialog).getByText('java.lang.OutOfMemoryError: Java heap space'),
    ).toBeInTheDocument();
    expect(within(dialog).getByText('4,096 lines')).toBeInTheDocument();

    await user.click(within(dialog).getByRole('button', { name: 'Save bundle' }));
    expect(
      await within(dialog).findByText('Written to D:/Mnelys/data/diagnostics/demo-1.md (2.0 KiB).'),
    ).toBeInTheDocument();
  });

  it('keeps the download queue alone', async () => {
    renderAt('/downloads', <DownloadsRoute />);
    expect(await screen.findByText('client.jar')).toBeInTheDocument();
    expect(screen.queryByText('Java runtimes')).not.toBeInTheDocument();
    expect(screen.queryByText('Minecraft versions')).not.toBeInTheDocument();
  });

  it('moves java into settings and the manifest into the developer page', async () => {
    renderAt('/settings', <SettingsRoute />);
    expect(await screen.findByText('Eclipse Adoptium')).toBeInTheDocument();

    renderAt('/developer', <DeveloperRoute />);
    expect(await screen.findByText('25w01a')).toBeInTheDocument();
  });

  it('renders the content hub as a result grid without the installed block', async () => {
    const user = userEvent.setup();
    renderAt('/content?instance=demo', <ContentRoute />);
    const search = await screen.findByLabelText('Search');
    expect(screen.queryByText('Installed content')).not.toBeInTheDocument();

    await user.type(search, 'sodium');
    await user.click(screen.getByRole('button', { name: 'Search' }));

    expect(await screen.findByText('Sodium')).toBeInTheDocument();
    expect(screen.getByText('jellysquid3')).toBeInTheDocument();
    expect(screen.getByText('32,145,678 downloads')).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: 'Install into' })).toHaveValue('demo');
  });
});
