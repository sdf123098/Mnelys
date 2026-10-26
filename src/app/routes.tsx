import { Navigate } from 'react-router-dom';
import type { RouteObject } from 'react-router-dom';

import { AppShell } from '@/components/AppShell';
import { AccountsRoute } from '@/routes/AccountsRoute';
import { ContentRoute } from '@/routes/ContentRoute';
import { DeveloperRoute } from '@/routes/DeveloperRoute';
import { DownloadsRoute } from '@/routes/DownloadsRoute';
import { InstancesRoute } from '@/routes/InstancesRoute';
import { MultiplayerRoute } from '@/routes/MultiplayerRoute';
import { SettingsRoute } from '@/routes/SettingsRoute';
import { StartRoute } from '@/routes/StartRoute';

/**
 * The one route table (ADR 0006).
 *
 * It lives beside the router rather than inside it so that `router.tsx` exports a
 * component and nothing else — a file that exports both a component and a value cannot
 * be hot-reloaded reliably, and the lint rule that guards that is not worth weakening.
 */
export const routes: RouteObject[] = [
  {
    path: '/',
    element: <AppShell />,
    children: [
      { index: true, element: <Navigate to="/start" replace /> },
      { path: 'start', element: <StartRoute /> },
      { path: 'instances', element: <InstancesRoute /> },
      // The same workspace, opened on one instance. The path segment is the selection.
      { path: 'instances/:id', element: <InstancesRoute /> },
      { path: 'downloads', element: <DownloadsRoute /> },
      { path: 'content', element: <ContentRoute /> },
      { path: 'multiplayer', element: <MultiplayerRoute /> },
      { path: 'developer', element: <DeveloperRoute /> },
      { path: 'accounts', element: <AccountsRoute /> },
      { path: 'settings', element: <SettingsRoute /> },
      { path: '*', element: <Navigate to="/start" replace /> },
    ],
  },
];
