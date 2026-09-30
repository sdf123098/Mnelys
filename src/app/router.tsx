import { createHashRouter, RouterProvider } from 'react-router-dom';

import { routes } from './routes';

/**
 * A hash router is used deliberately (ADR 0006).
 *
 * In a packaged Tauri build the frontend is served from a custom protocol, and not every
 * platform's handler rewrites an unknown path to `index.html`; a hash route survives a
 * restart under all three webviews, which is the property that matters for deep links.
 */
const router = createHashRouter(routes);

export function AppRouter() {
  return <RouterProvider router={router} />;
}
