import { Navigate } from 'react-router-dom';
import type { RouteObject } from 'react-router-dom';

import { AppShell } from '@/components/AppShell';
import { AccountsRoute } from '@/routes/AccountsRoute';
import { InstancesRoute } from '@/routes/InstancesRoute';
import { SettingsRoute } from '@/routes/SettingsRoute';
import { TasksRoute } from '@/routes/TasksRoute';

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
      { index: true, element: <Navigate to="/instances" replace /> },
      { path: 'instances', element: <InstancesRoute /> },
      { path: 'accounts', element: <AccountsRoute /> },
      { path: 'tasks', element: <TasksRoute /> },
      { path: 'settings', element: <SettingsRoute /> },
      { path: '*', element: <Navigate to="/instances" replace /> },
    ],
  },
];
