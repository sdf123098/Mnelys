import { useTranslation } from 'react-i18next';

import type { TaskSnapshot } from '@/ipc/commands';
import { Badge, type BadgeTone } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';

import styles from './TaskList.module.css';

const STATE_TONES: Readonly<Record<TaskSnapshot['state'], BadgeTone>> = {
  queued: 'neutral',
  running: 'info',
  succeeded: 'success',
  failed: 'danger',
  cancelled: 'warning',
};

function isActive(task: TaskSnapshot): boolean {
  return task.state === 'queued' || task.state === 'running';
}

interface TaskListProps {
  readonly tasks: readonly TaskSnapshot[];
  readonly onCancel?: (id: string) => void;
  readonly cancelling?: boolean;
}

/** The one task list, shared by the task center, downloads and developer pages. */
export function TaskList({ tasks, onCancel, cancelling = false }: TaskListProps) {
  const { t } = useTranslation();

  return (
    <ul className={styles.list}>
      {tasks.map((task) => {
        const total = task.progress.total;
        const percent =
          total === undefined || total === 0
            ? undefined
            : Math.min(100, Math.round((task.progress.current / total) * 100));

        return (
          <li key={task.id} className={styles.item}>
            <div className={styles.main}>
              <div className={styles.titleRow}>
                <span className={styles.label}>{task.label}</span>
                <Badge tone={STATE_TONES[task.state]}>{t(`tasks.state.${task.state}`)}</Badge>
              </div>
              <div
                className={styles.progress}
                role="progressbar"
                aria-valuemin={0}
                aria-valuemax={100}
                aria-valuenow={percent}
              >
                <span
                  className={
                    percent === undefined ? `${styles.fill} ${styles.indeterminate}` : styles.fill
                  }
                  style={percent === undefined ? undefined : { width: `${percent}%` }}
                />
              </div>
              <span className={styles.message}>
                {task.progress.message.length === 0
                  ? t('tasks.stateMessage', { state: t(`tasks.state.${task.state}`) })
                  : task.progress.message}
                {percent === undefined ? '' : ` · ${percent}%`}
              </span>
              {task.error === undefined ? null : <span className={styles.error}>{task.error}</span>}
            </div>
            {onCancel === undefined || !isActive(task) ? null : (
              <Button
                size="sm"
                variant="ghost"
                icon="close"
                disabled={cancelling}
                onClick={() => {
                  onCancel(task.id);
                }}
              >
                {t('tasks.cancel')}
              </Button>
            )}
          </li>
        );
      })}
    </ul>
  );
}
