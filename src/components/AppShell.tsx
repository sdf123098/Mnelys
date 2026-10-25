import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Outlet, useLocation } from 'react-router-dom';

import styles from './AppShell.module.css';
import { NavSidebar } from './NavSidebar';
import { StatusBar } from './StatusBar';
import { TaskCenterDrawer } from './TaskCenterDrawer';
import { TopBar } from './TopBar';

/**
 * Window chrome: navigation, routed content, status bar, task overlay.
 *
 * The shell owns no product data. Everything it displays about the core comes from
 * TanStack Query through `src/ipc/hooks.ts`, so the Rust core stays the source of truth.
 */
export function AppShell() {
  const { t } = useTranslation();
  const { pathname } = useLocation();
  const contentRef = useRef<HTMLElement>(null);

  useEffect(() => {
    if (contentRef.current !== null) {
      contentRef.current.scrollTop = 0;
    }
  }, [pathname]);

  return (
    <div className={styles.shell}>
      <a
        className={styles.skipLink}
        href="#main-content"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById('main-content')?.focus();
        }}
      >
        {t('common.skipToContent')}
      </a>
      <NavSidebar />
      <TopBar />
      <main id="main-content" ref={contentRef} className={styles.content} tabIndex={-1}>
        <div className={styles.contentInner}>
          <Outlet />
        </div>
      </main>
      <StatusBar />
      <TaskCenterDrawer />
    </div>
  );
}
