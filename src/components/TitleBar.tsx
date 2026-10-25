import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useRuntimeInfo } from '@/ipc/hooks';
import {
  closeWindow,
  detectPlatform,
  isWindowMaximized,
  minimizeWindow,
  onWindowResized,
  setWindowTitle,
  toggleMaximizeWindow,
} from '@/ipc/window';
import { Icon } from './ui/Icon';

import styles from './TitleBar.module.css';

/**
 * A drawn title bar for Windows and Linux, and an overlay for macOS.
 *
 * macOS keeps its native traffic lights, so the bar only reserves their space and draws
 * no controls. The locale change is pushed to the OS title as well, which keeps the
 * taskbar and window switcher showing the same single product name as the UI.
 */
export function TitleBar() {
  const { t, i18n } = useTranslation();
  const runtime = useRuntimeInfo();
  const platform = runtime.data?.os ?? detectPlatform();
  const macos = platform === 'macos';
  const [maximized, setMaximized] = useState(false);

  useEffect(() => {
    document.documentElement.dataset['os'] = platform;
  }, [platform]);

  useEffect(() => {
    const name = t('app.name');
    document.title = name;
    void setWindowTitle(name).catch(() => undefined);
  }, [t, i18n.resolvedLanguage]);

  useEffect(() => {
    if (macos) {
      return undefined;
    }
    let active = true;
    const sync = (): void => {
      void isWindowMaximized()
        .then((value) => {
          if (active) {
            setMaximized(value);
          }
        })
        .catch(() => undefined);
    };
    sync();
    let stop: (() => void) | undefined;
    void onWindowResized(sync)
      .then((unlisten) => {
        if (active) {
          stop = unlisten;
        } else {
          unlisten();
        }
      })
      .catch(() => undefined);
    return () => {
      active = false;
      stop?.();
    };
  }, [macos]);

  function run(action: () => Promise<void>): void {
    void action().catch(() => undefined);
  }

  return (
    <div
      className={styles.bar}
      data-tauri-drag-region
      onDoubleClick={(event) => {
        if (!macos && (event.target as HTMLElement).closest('button') === null) {
          run(toggleMaximizeWindow);
        }
      }}
    >
      <div className={styles.spacer} data-tauri-drag-region />

      {macos ? null : (
        <div className={styles.controls}>
          <button
            type="button"
            className={styles.control}
            title={t('window.minimize')}
            aria-label={t('window.minimize')}
            onClick={() => {
              run(minimizeWindow);
            }}
          >
            <Icon name="minimize" size={14} />
          </button>
          <button
            type="button"
            className={styles.control}
            title={maximized ? t('window.restore') : t('window.maximize')}
            aria-label={maximized ? t('window.restore') : t('window.maximize')}
            onClick={() => {
              run(toggleMaximizeWindow);
            }}
          >
            <Icon name={maximized ? 'restore' : 'maximize'} size={maximized ? 12 : 13} />
          </button>
          <button
            type="button"
            className={`${styles.control} ${styles.close}`}
            title={t('window.close')}
            aria-label={t('window.close')}
            onClick={() => {
              run(closeWindow);
            }}
          >
            <Icon name="close" size={15} />
          </button>
        </div>
      )}
    </div>
  );
}
