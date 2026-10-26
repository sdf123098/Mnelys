import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { PageHeader } from '@/components/ui/PageHeader';
import { TaskList } from '@/components/TaskList';
import type { TaskSnapshot } from '@/ipc/commands';
import { useCancelTask, useTasks } from '@/ipc/hooks';

import pages from './pages.module.css';

function isActive(task: TaskSnapshot): boolean {
  return task.state === 'queued' || task.state === 'running';
}

export function DownloadsRoute() {
  const { t } = useTranslation();
  const tasks = useTasks(true);
  const cancelTask = useCancelTask();

  const all = tasks.data ?? [];
  const activeTasks = all.filter(isActive);
  const recentTasks = all.filter((task) => !isActive(task));

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
        {all.length === 0 ? (
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
    </div>
  );
}
