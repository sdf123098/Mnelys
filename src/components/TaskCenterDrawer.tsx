import { useEffect, useId, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Badge, type BadgeTone } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Dialog } from '@/components/ui/Dialog';
import { Icon } from '@/components/ui/Icon';
import { IconButton } from '@/components/ui/IconButton';
import { SegmentedControl, type SegmentOption } from '@/components/ui/SegmentedControl';
import type { AppLogEntry, LogLevel } from '@/ipc/commands';
import { useAppLogs, useClearAppLogs } from '@/ipc/hooks';
import { useUiStore } from '@/state/ui-store';

import styles from './TaskCenterDrawer.module.css';

const FOCUSABLE =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

type TaskCenterTab = 'tasks' | 'logs';

const LEVEL_TONES: Readonly<Record<LogLevel, BadgeTone>> = {
  trace: 'neutral',
  debug: 'neutral',
  info: 'info',
  warn: 'warning',
  error: 'danger',
};

const LEVEL_KEYS = {
  trace: 'tasks.logs.level.trace',
  debug: 'tasks.logs.level.debug',
  info: 'tasks.logs.level.info',
  warn: 'tasks.logs.level.warn',
  error: 'tasks.logs.level.error',
} as const satisfies Readonly<Record<LogLevel, string>>;

/**
 * The task center is an overlay rather than a route. It stays inside the window, keeps
 * the current page visible, traps focus, closes on Escape and restores focus on exit.
 *
 * It has two sections: the future task list and the core's live in-memory log. The log
 * is the first real core-owned data the shell displays; it is polled only while the
 * section is visible.
 */
export function TaskCenterDrawer() {
  const { t, i18n } = useTranslation();
  const open = useUiStore((state) => state.taskCenterOpen);
  const closeTaskCenter = useUiStore((state) => state.closeTaskCenter);

  const [tab, setTab] = useState<TaskCenterTab>('tasks');
  const [confirmClear, setConfirmClear] = useState(false);
  const confirmClearRef = useRef(false);

  const logsQuery = useAppLogs(open && tab === 'logs');
  const clearLogs = useClearAppLogs();

  const titleId = useId();
  const descriptionId = useId();
  const drawerRef = useRef<HTMLElement>(null);
  const closeButtonRef = useRef<HTMLButtonElement>(null);

  const formatTime = useMemo(() => {
    const formatter = new Intl.DateTimeFormat(i18n.language, {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
    return (timestampMs: number) => formatter.format(timestampMs);
  }, [i18n.language]);

  const tabOptions: readonly SegmentOption<TaskCenterTab>[] = [
    { value: 'tasks', label: t('tasks.tabs.tasks'), icon: 'tasks' },
    { value: 'logs', label: t('tasks.tabs.logs'), icon: 'terminal' },
  ];

  function dismissConfirm(): void {
    setConfirmClear(false);
    confirmClearRef.current = false;
    clearLogs.reset();
  }

  function closeDrawer(): void {
    dismissConfirm();
    closeTaskCenter();
  }

  function requestClear(): void {
    clearLogs.reset();
    setConfirmClear(true);
    confirmClearRef.current = true;
  }

  function confirmClearLogs(): void {
    clearLogs.mutate(undefined, {
      onSuccess: () => {
        setConfirmClear(false);
        confirmClearRef.current = false;
      },
    });
  }

  useEffect(() => {
    if (!open) {
      return;
    }

    const previouslyFocused =
      document.activeElement instanceof HTMLElement ? document.activeElement : null;
    closeButtonRef.current?.focus();

    function onKeyDown(event: KeyboardEvent): void {
      if (event.key === 'Escape') {
        // The confirmation is a native modal dialog; let it cancel itself.
        if (confirmClearRef.current) {
          return;
        }
        event.stopPropagation();
        setConfirmClear(false);
        confirmClearRef.current = false;
        closeTaskCenter();
        return;
      }
      if (event.key !== 'Tab') {
        return;
      }

      const drawer = drawerRef.current;
      if (drawer === null) {
        return;
      }
      const focusable = Array.from(drawer.querySelectorAll<HTMLElement>(FOCUSABLE));
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (first === undefined || last === undefined) {
        return;
      }

      if (event.shiftKey && document.activeElement === first) {
        event.preventDefault();
        last.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first.focus();
      }
    }

    document.addEventListener('keydown', onKeyDown, true);
    return () => {
      document.removeEventListener('keydown', onKeyDown, true);
      previouslyFocused?.focus();
    };
  }, [open, closeTaskCenter]);

  if (!open) {
    return null;
  }

  const entries = logsQuery.data;
  const clearError = clearLogs.isError ? (
    <p className={styles.dialogError} role="alert">
      {t('tasks.logs.clearError')}
    </p>
  ) : undefined;

  return (
    <div className={styles.layer}>
      <div className={styles.scrim} aria-hidden="true" onClick={closeDrawer} />
      <section
        ref={drawerRef}
        className={styles.drawer}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={descriptionId}
      >
        <header className={styles.header}>
          <span className={styles.headerIcon} aria-hidden="true">
            <Icon name="tasks" size={18} />
          </span>
          <div className={styles.heading}>
            <h2 id={titleId}>{t('tasks.title')}</h2>
            <p id={descriptionId}>{t('tasks.description')}</p>
          </div>
          <IconButton
            ref={closeButtonRef}
            icon="close"
            label={t('tasks.close')}
            onClick={closeDrawer}
          />
        </header>

        <div className={styles.tabs}>
          <SegmentedControl
            name="task-center-section"
            label={t('tasks.tabs.label')}
            value={tab}
            options={tabOptions}
            onChange={setTab}
          />
        </div>

        <div className={tab === 'logs' ? `${styles.body} ${styles.bodyLogs}` : styles.body}>
          {tab === 'tasks' ? (
            <div className={styles.empty}>
              <span className={styles.emptyIcon} aria-hidden="true">
                <Icon name="tasks" size={26} strokeWidth={1.5} />
              </span>
              <h3>{t('tasks.emptyTitle')}</h3>
              <p>{t('tasks.emptyBody')}</p>
            </div>
          ) : (
            <LogsPanel
              pending={logsQuery.isPending}
              failed={logsQuery.isError}
              entries={entries}
              formatTime={formatTime}
              onRetry={() => {
                void logsQuery.refetch();
              }}
            />
          )}
        </div>

        <footer className={styles.footer}>
          {tab === 'tasks' ? (
            <Badge tone="neutral" icon="info">
              {t('tasks.footer')}
            </Badge>
          ) : (
            <>
              <Badge tone="neutral" icon="terminal">
                {t('tasks.logs.count', { count: entries?.length ?? 0 })}
              </Badge>
              <span className={styles.footerSpacer} />
              <Button
                size="sm"
                variant="ghost"
                icon="trash"
                onClick={requestClear}
                disabled={clearLogs.isPending}
              >
                {t('tasks.logs.clear')}
              </Button>
            </>
          )}
        </footer>
      </section>

      <Dialog
        open={confirmClear}
        title={t('tasks.logs.confirmTitle')}
        description={t('tasks.logs.confirmBody')}
        closeLabel={t('tasks.logs.confirmCancel')}
        onClose={dismissConfirm}
        footer={
          <>
            <Button size="sm" variant="secondary" onClick={dismissConfirm}>
              {t('tasks.logs.confirmCancel')}
            </Button>
            <Button
              size="sm"
              variant="danger"
              icon="trash"
              onClick={confirmClearLogs}
              disabled={clearLogs.isPending}
            >
              {t('tasks.logs.confirmAction')}
            </Button>
          </>
        }
        children={clearError}
      />
    </div>
  );
}

interface LogsPanelProps {
  readonly pending: boolean;
  readonly failed: boolean;
  readonly entries: readonly AppLogEntry[] | undefined;
  readonly formatTime: (timestampMs: number) => string;
  readonly onRetry: () => void;
}

function LogsPanel({ pending, failed, entries, formatTime, onRetry }: LogsPanelProps) {
  const { t } = useTranslation();

  if (pending) {
    return (
      <p className={styles.logStatus} role="status">
        <Icon name="loading" size={16} />
        {t('tasks.logs.loading')}
      </p>
    );
  }

  if (failed) {
    return (
      <div className={styles.empty}>
        <span className={styles.emptyIcon} aria-hidden="true">
          <Icon name="statusError" size={26} strokeWidth={1.5} />
        </span>
        <h3>{t('tasks.logs.errorTitle')}</h3>
        <p>{t('tasks.logs.errorBody')}</p>
        <Button size="sm" variant="secondary" icon="refresh" onClick={onRetry}>
          {t('error.retry')}
        </Button>
      </div>
    );
  }

  if (entries === undefined || entries.length === 0) {
    return (
      <div className={styles.empty}>
        <span className={styles.emptyIcon} aria-hidden="true">
          <Icon name="terminal" size={26} strokeWidth={1.5} />
        </span>
        <h3>{t('tasks.logs.emptyTitle')}</h3>
        <p>{t('tasks.logs.emptyBody')}</p>
      </div>
    );
  }

  return (
    <ul className={styles.logList}>
      {entries.map((entry) => (
        <li key={entry.id} className={styles.logItem}>
          <div className={styles.logMeta}>
            <time className={styles.logTime} dateTime={new Date(entry.timestampMs).toISOString()}>
              {formatTime(entry.timestampMs)}
            </time>
            <Badge tone={LEVEL_TONES[entry.level]}>{t(LEVEL_KEYS[entry.level])}</Badge>
            <span className={styles.logTarget}>{entry.target}</span>
          </div>
          <p className={styles.logMessage}>{entry.message}</p>
        </li>
      ))}
    </ul>
  );
}
