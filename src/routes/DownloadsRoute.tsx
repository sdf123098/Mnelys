import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { TaskList } from '@/components/TaskList';
import {
  useCancelTask,
  useContentProviders,
  useInstallJava,
  useJavaRuntimes,
  useTasks,
  useVersions,
} from '@/ipc/hooks';

import pages from './pages.module.css';

const JAVA_MAJORS: readonly number[] = [8, 17, 21];

export function DownloadsRoute() {
  const { t } = useTranslation();
  const versions = useVersions();
  const java = useJavaRuntimes();
  const installJava = useInstallJava();
  const providers = useContentProviders();
  const tasks = useTasks(true);
  const cancelTask = useCancelTask();

  const [filter, setFilter] = useState('');

  const filteredVersions = useMemo(() => {
    const all = versions.data?.versions ?? [];
    const needle = filter.trim().toLowerCase();
    const matches =
      needle.length === 0
        ? all
        : all.filter((version) => version.id.toLowerCase().includes(needle));
    return matches.slice(0, 200);
  }, [versions.data, filter]);

  const activeTasks = (tasks.data ?? []).filter(
    (task) => task.state === 'queued' || task.state === 'running',
  );
  const recentTasks = (tasks.data ?? []).filter(
    (task) => task.state !== 'queued' && task.state !== 'running',
  );

  return (
    <div className={pages.page}>
      <PageHeader title={t('downloads.heading')} description={t('downloads.description')} />

      <Card
        title={t('downloads.queueTitle')}
        description={t('downloads.queueBody')}
        icon="tasks"
        badge={
          <Badge tone={activeTasks.length > 0 ? 'info' : 'neutral'}>
            {t('downloads.activeTasks', { count: activeTasks.length })}
          </Badge>
        }
      >
        {(tasks.data ?? []).length === 0 ? (
          <EmptyState
            icon="downloads"
            title={t('downloads.emptyTitle')}
            description={t('downloads.emptyBody')}
          />
        ) : (
          <>
            {activeTasks.length === 0 ? null : (
              <TaskList
                tasks={activeTasks}
                cancelling={cancelTask.isPending}
                onCancel={(id) => {
                  cancelTask.mutate(id);
                }}
              />
            )}
            {recentTasks.length === 0 ? null : (
              <>
                <h3 className={pages.sectionHeader}>{t('downloads.recentTitle')}</h3>
                <TaskList tasks={recentTasks} />
              </>
            )}
          </>
        )}
      </Card>

      <div className={pages.grid}>
        <Card
          title={t('downloads.javaTitle')}
          description={t('downloads.javaBody')}
          icon="java"
          badge={
            <Badge tone="neutral">
              {t('downloads.javaDetected', { count: java.data?.length ?? 0 })}
            </Badge>
          }
        >
          <div className={pages.wideBlock}>
            <div className={pages.inlineForm}>
              {JAVA_MAJORS.map((major) => (
                <Button
                  key={major}
                  size="sm"
                  variant="secondary"
                  icon="downloads"
                  disabled={installJava.isPending}
                  onClick={() => {
                    installJava.mutate(major);
                  }}
                >
                  Java {major}
                </Button>
              ))}
            </div>
            <p className={pages.muted}>{t('downloads.javaInstallHint')}</p>
            {java.isError ? <p className={pages.errorText}>{t('downloads.javaFailed')}</p> : null}
            {java.data !== undefined && java.data.length === 0 ? (
              <p className={pages.muted}>{t('downloads.javaEmpty')}</p>
            ) : null}
            <ul className={pages.list}>
              {java.data?.map((runtime) => (
                <li key={runtime.path} className={pages.row}>
                  <span className={pages.rowMeta} aria-hidden="true">
                    <Icon name="java" size={18} />
                  </span>
                  <div className={pages.rowMain}>
                    <span className={pages.rowTitle}>
                      Java {runtime.major} · {runtime.version}
                    </span>
                    <span className={pages.rowMeta}>{runtime.path}</span>
                  </div>
                  <Badge tone={runtime.source === 'managed' ? 'accent' : 'neutral'}>
                    {t(`downloads.javaSource.${runtime.source}`)}
                  </Badge>
                  <span className={pages.rowMeta}>{runtime.vendor}</span>
                </li>
              ))}
            </ul>
          </div>
        </Card>

        <Card
          title={t('downloads.versionsTitle')}
          description={t('downloads.versionsBody')}
          icon="database"
          badge={
            <Badge tone="neutral">
              {t('downloads.versionsCount', { count: versions.data?.versions.length ?? 0 })}
            </Badge>
          }
        >
          <div className={pages.wideBlock}>
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="version-filter">
                {t('downloads.versionsFilter')}
              </label>
              <input
                id="version-filter"
                className={pages.input}
                value={filter}
                spellCheck={false}
                onChange={(event) => {
                  setFilter(event.target.value);
                }}
              />
            </div>
            {versions.isPending ? (
              <p className={pages.muted} role="status">
                <Icon name="loading" size={16} /> {t('downloads.versionsLoading')}
              </p>
            ) : null}
            {versions.isError ? (
              <p className={pages.errorText}>{t('downloads.versionsFailed')}</p>
            ) : null}
            <ul className={`${pages.list} ${pages.scrollList}`}>
              {filteredVersions.map((version) => (
                <li key={version.id} className={pages.row}>
                  <div className={pages.rowMain}>
                    <span className={pages.rowTitle}>{version.id}</span>
                    <span className={pages.rowMeta}>
                      {version.releaseTime === undefined
                        ? t('downloads.noReleaseTime')
                        : new Date(version.releaseTime).toLocaleDateString()}
                    </span>
                  </div>
                  <Badge tone={version.type === 'release' ? 'success' : 'neutral'}>
                    {t(`downloads.versionType.${version.type}`)}
                  </Badge>
                </li>
              ))}
            </ul>
            {versions.data !== undefined && filteredVersions.length === 0 ? (
              <p className={pages.muted}>{t('downloads.versionsEmpty')}</p>
            ) : null}
          </div>
        </Card>
      </div>

      <Card
        title={t('downloads.providersHeading')}
        description={t('downloads.providersBody')}
        icon="globe"
      >
        <ul className={pages.list}>
          <li className={pages.row}>
            <span className={pages.rowMeta} aria-hidden="true">
              <Icon name="globe" size={18} />
            </span>
            <div className={pages.rowMain}>
              <span className={pages.rowTitle}>{t('downloads.contentPlatforms')}</span>
              <span className={pages.rowMeta}>
                {providers.data === undefined
                  ? t('downloads.providersLoading')
                  : providers.data.map((id) => t(`contentProvider.${id}`)).join(' · ')}
              </span>
            </div>
            <ButtonLink to="/instances" variant="ghost" size="sm" iconRight="arrowRight">
              {t('downloads.providersAction')}
            </ButtonLink>
          </li>
        </ul>
        <p className={pages.caption}>{t('downloads.providersNote')}</p>
      </Card>
    </div>
  );
}
