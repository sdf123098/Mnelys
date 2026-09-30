import { useEffect } from 'react';
import { Outlet, useLocation } from 'react-router-dom';

import { useUiStore } from '@/state/ui-store';

import styles from './AppShell.module.css';
import { NavSidebar } from './NavSidebar';
import { StatusBar } from './StatusBar';

/**
 * Window chrome: navigation, routed content, status bar.
 *
 * The shell owns no product data. Everything it displays about the core comes from
 * TanStack Query through `src/ipc/hooks.ts`, so the Rust core stays the source of truth.
 */
export function AppShell() {
  const { pathname } = useLocation();
  const setLastNavSection = useUiStore((state) => state.setLastNavSection);

  useEffect(() => {
    const [section] = pathname.split('/').filter(Boolean);
    setLastNavSection(section ?? 'instances');
  }, [pathname, setLastNavSection]);

  return (
    <div className={styles.shell}>
      <NavSidebar />
      <main className={styles.content}>
        <div className={styles.contentInner}>
          <Outlet />
        </div>
      </main>
      <StatusBar />
    </div>
  );
}
