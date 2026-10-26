import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { VERSION_TYPE_KEYS } from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { TaskList } from '@/components/TaskList';
import { useCancelTask, useTasks, useVersions } from '@/ipc/hooks';
import { usePreferences } from '@/state/preferences-store';
import { useUiStore } from '@/state/ui-store';

import pages from './pages.module.css';

/** One manifest page is enough for inspection; the installer reads the full list itself. */
const MANIFEST_LIMIT = 200;

const TOOLS = [
  {
    id: 'build',
    icon: 'terminal',
    titleKey: 'developer.tools.build.title',
    bodyKey: 'developer.tools.build.body',
  },
  {
    id: 'debug',
    icon: 'bug',
    titleKey: 'developer.tools.debug.title',
    bodyKey: 'developer.tools.debug.body',
  },
  {
    id: 'daemon',
    icon: 'server',
    titleKey: 'developer.tools.daemon.title',
    bodyKey: 'developer.tools.daemon.body',
  },
] as const;

export function DeveloperRoute() {
  const { t } = useTranslation();
  const developerMode = usePreferences((state) => state.developerMode);
  const setDeveloperMode = usePreferences((state) => state.setDeveloperMode);
  const openTaskCenter = useUiStore((state) => state.openTaskCenter);
  const tasks = useTasks(developerMode);
  const cancelTask = useCancelTask();

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('developer.heading')}
        description={t('developer.description')}
        badge={
          <Badge tone={developerMode ? 'success' : 'neutral'}>
            {developerMode ? t('developer.enabledTitle') : t('developer.gateTitle')}
          </Badge>
        }
        actions={
          developerMode ? (
            <Button
              variant="secondary"
              icon="close"
              onClick={() => {
                setDeveloperMode(false);
              }}
            >
              {t('developer.disableAction')}
            </Button>
          ) : undefined
        }
      />

      {developerMode ? (
        <>
          <header className={pages.sectionHeader}>
            <h2>{t('developer.toolsTitle')}</h2>
            <p>{t('developer.enabledBody')}</p>
          </header>
          <div className={pages.grid}>
            {TOOLS.map((tool) => (
              <Card
                key={tool.id}
                icon={tool.icon}
                title={t(tool.titleKey)}
                description={t(tool.bodyKey)}
              />
            ))}
          </div>

          <Card
            title={t('developer.tasksTitle')}
            description={t('developer.tasksBody')}
            icon="tasks"
            badge={
              <Badge tone="neutral">
                {t('developer.tasksCount', { count: tasks.data?.length ?? 0 })}
              </Badge>
            }
            actions={
              <Button size="sm" variant="ghost" icon="terminal" onClick={openTaskCenter}>
                {t('tasks.title')}
              </Button>
            }
          >
            {tasks.data !== undefined && tasks.data.length === 0 ? (
              <p className={pages.muted}>{t('tasks.emptyTitle')}</p>
            ) : (
              <TaskList
                tasks={tasks.data ?? []}
                cancelling={cancelTask.isPending}
                onCancel={(id) => {
                  cancelTask.mutate(id);
                }}
              />
            )}
          </Card>

          <VersionManifestCard />
        </>
      ) : (
        <Card
          icon="developer"
          title={t('developer.gateTitle')}
          description={t('developer.gateBody')}
          actions={
            <Button
              variant="primary"
              icon="check"
              onClick={() => {
                setDeveloperMode(true);
              }}
            >
              {t('developer.gateAction')}
            </Button>
          }
        />
      )}
    </div>
  );
}

/** Mojang's raw manifest, read straight from the core so a mirror swap can be inspected. */
function VersionManifestCard() {
  const { t } = useTranslation();
  const versions = useVersions();
  const [filter, setFilter] = useState('');

  const rows = useMemo(() => {
    const all = versions.data?.versions ?? [];
    const needle = filter.trim().toLowerCase();
    const matches =
      needle.length === 0 ? all : all.filter((entry) => entry.id.toLowerCase().includes(needle));
    return matches.slice(0, MANIFEST_LIMIT);
  }, [versions.data, filter]);

  return (
    <Card
      title={t('developer.manifestTitle')}
      description={t('developer.manifestBody')}
      icon="database"
      badge={
        <Badge tone="neutral">
          {t('developer.manifestCount', { count: versions.data?.versions.length ?? 0 })}
        </Badge>
      }
    >
      <div className={pages.wideBlock}>
        <div className={pages.field}>
          <label className={pages.fieldLabel} htmlFor="manifest-filter">
            {t('developer.manifestFilter')}
          </label>
          <input
            id="manifest-filter"
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
            <Icon name="loading" size={16} /> {t('developer.manifestLoading')}
          </p>
        ) : null}
        {versions.isError ? (
          <p className={pages.errorText}>{t('developer.manifestFailed')}</p>
        ) : null}
        <ul className={`${pages.list} ${pages.scrollList}`}>
          {rows.map((entry) => (
            <li key={entry.id} className={pages.row}>
              <div className={pages.rowMain}>
                <span className={pages.rowTitle}>{entry.id}</span>
                <span className={pages.rowMeta}>
                  {entry.releaseTime === undefined
                    ? t('version.noReleaseTime')
                    : new Date(entry.releaseTime).toLocaleDateString()}
                </span>
              </div>
              <Badge tone={entry.type === 'release' ? 'success' : 'neutral'}>
                {t(VERSION_TYPE_KEYS[entry.type])}
              </Badge>
            </li>
          ))}
        </ul>
        {versions.data !== undefined && rows.length === 0 ? (
          <p className={pages.muted}>{t('developer.manifestEmpty')}</p>
        ) : null}
      </div>
    </Card>
  );
}
