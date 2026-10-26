import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { InstanceEntryList, type EntryListItem } from '@/components/InstanceEntryList';
import { Button } from '@/components/ui/Button';
import { useInstanceLogTail, useInstanceLogs, useLaunchLogs } from '@/ipc/hooks';

import styles from './InstanceSettingsDialog.module.css';
import pages from '@/routes/pages.module.css';

interface InstanceLogsPanelProps {
  readonly instanceId: string;
  /** Newest session of this instance; the live pane only exists while one is known. */
  readonly sessionId: string | undefined;
}

/**
 * Two log sources, because they answer different questions.
 *
 * The core buffer is what the running game prints right now; the files below `logs/`
 * and `crash-reports/` are what survived the last exits. The file list is read straight
 * from disk, so a crash the game wrote after the process died is still reachable.
 */
export function InstanceLogsPanel({ instanceId, sessionId }: InstanceLogsPanelProps) {
  const { t } = useTranslation();
  const files = useInstanceLogs(instanceId, true);
  const live = useLaunchLogs(sessionId, sessionId !== undefined);
  const [openPath, setOpenPath] = useState<string | undefined>(undefined);
  const tail = useInstanceLogTail(instanceId, openPath);

  const logs: EntryListItem[] = (files.data?.logs ?? []).map((entry) => ({ ...entry }));
  const crashes: EntryListItem[] = (files.data?.crashReports ?? []).map((entry) => ({
    ...entry,
    icon: 'bug',
  }));

  return (
    <>
      {sessionId === undefined ? (
        <p className={pages.muted}>{t('instances.noLogs')}</p>
      ) : (
        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>{t('instances.logsLiveTitle')}</h3>
          <p className={pages.muted}>{t('instances.logsBody')}</p>
          {live.isError ? (
            <p className={pages.errorText}>{t('instances.logsFailed')}</p>
          ) : (live.data ?? []).length === 0 ? (
            <p className={pages.muted}>{t('instances.logsEmpty')}</p>
          ) : (
            <pre className={`${pages.codeBox} ${styles.log}`}>{(live.data ?? []).join('\n')}</pre>
          )}
        </section>
      )}

      <section className={styles.section}>
        <h3 className={styles.sectionTitle}>{t('instances.logsFilesTitle')}</h3>
        <p className={pages.muted}>{t('instances.logsFilesBody')}</p>
        {files.isError ? (
          <p className={pages.errorText}>{t('instances.entriesUnavailable')}</p>
        ) : files.isPending ? (
          <p className={pages.muted}>{t('common.loading')}</p>
        ) : crashes.length === 0 && logs.length === 0 ? (
          <p className={pages.muted}>{t('instances.logsFileEmpty')}</p>
        ) : (
          <>
            {crashes.length === 0 ? null : (
              <section className={styles.subList} aria-label={t('instances.logsCrashTitle')}>
                <h4 className={styles.subTitle}>{t('instances.logsCrashTitle')}</h4>
                <InstanceEntryList
                  instanceId={instanceId}
                  items={crashes}
                  open={{
                    label: t('instances.logsViewFile'),
                    onSelect: (item) => {
                      setOpenPath(item.path);
                    },
                  }}
                  empty={null}
                />
              </section>
            )}
            {logs.length === 0 ? null : (
              <section className={styles.subList} aria-label={t('instances.logsLogTitle')}>
                <h4 className={styles.subTitle}>{t('instances.logsLogTitle')}</h4>
                <InstanceEntryList
                  instanceId={instanceId}
                  items={logs}
                  open={{
                    label: t('instances.logsViewFile'),
                    onSelect: (item) => {
                      setOpenPath(item.path);
                    },
                  }}
                  empty={null}
                />
              </section>
            )}
          </>
        )}
      </section>

      {openPath === undefined ? null : (
        <section className={styles.section}>
          <div className={styles.tailHead}>
            <h3 className={styles.sectionTitle}>{openPath}</h3>
            <Button
              size="sm"
              variant="ghost"
              icon="close"
              onClick={() => {
                setOpenPath(undefined);
              }}
            >
              {t('common.close')}
            </Button>
          </div>
          {tail.isError ? (
            <p className={pages.errorText}>{t('instances.logsFailed')}</p>
          ) : tail.isPending ? (
            <p className={pages.muted}>{t('common.loading')}</p>
          ) : tail.data === undefined || tail.data.text.length === 0 ? (
            <p className={pages.muted}>{t('instances.logsEmpty')}</p>
          ) : (
            <>
              <pre className={`${pages.codeBox} ${styles.log}`}>{tail.data.text}</pre>
              {tail.data.truncated ? (
                <p className={pages.muted}>{t('instances.logsTruncated')}</p>
              ) : null}
            </>
          )}
        </section>
      )}
    </>
  );
}
