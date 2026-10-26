import { useTranslation } from 'react-i18next';

import { CONTENT_KIND_KEYS, CONTENT_KIND_ORDER } from '@/app/wire-labels';
import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon, type IconName } from '@/components/ui/Icon';
import type { ContentKind, InstalledContent } from '@/ipc/commands';

import styles from './InstalledContentGroups.module.css';

const KIND_ICONS: Readonly<Record<ContentKind, IconName>> = {
  mod: 'package',
  resourcepack: 'layers',
  shader: 'sparkles',
  world: 'globe',
  datapack: 'database',
};

interface InstalledContentGroupsProps {
  readonly items: readonly InstalledContent[];
  readonly removeLabel: string;
  readonly disabled: boolean;
  readonly onRemove: (projectId: string) => void;
  readonly emptyTitle?: string;
  readonly emptyBody?: string;
}

/**
 * Installed content grouped by resource type, so a data pack never reads as a mod.
 * Both the content hub and the instance detail use this ordering.
 */
export function InstalledContentGroups({
  items,
  removeLabel,
  disabled,
  onRemove,
  emptyTitle,
  emptyBody,
}: InstalledContentGroupsProps) {
  const { t } = useTranslation();

  if (items.length === 0) {
    return emptyTitle === undefined ? null : (
      <EmptyState icon="package" title={emptyTitle} description={emptyBody ?? ''} />
    );
  }

  return (
    <div className={styles.groups}>
      {CONTENT_KIND_ORDER.map((kind) => {
        const group = items.filter((item) => item.kind === kind);
        if (group.length === 0) {
          return null;
        }

        return (
          <section key={kind} className={styles.group}>
            <h3 className={styles.groupTitle}>
              <Icon name={KIND_ICONS[kind]} size={16} />
              {t(CONTENT_KIND_KEYS[kind])}
              <span className={styles.groupCount}>{group.length}</span>
            </h3>
            <ul className={styles.list}>
              {group.map((item) => (
                <li key={`${item.provider}-${item.projectId}`} className={styles.row}>
                  <div className={styles.rowMain}>
                    <span className={styles.rowTitle}>{item.name}</span>
                    <span className={styles.rowMeta}>
                      {item.filename} · {t(`contentProvider.${item.provider}`)}
                    </span>
                  </div>
                  <Button
                    size="sm"
                    variant="danger"
                    icon="trash"
                    disabled={disabled}
                    onClick={() => {
                      onRemove(item.projectId);
                    }}
                  >
                    {removeLabel}
                  </Button>
                </li>
              ))}
            </ul>
          </section>
        );
      })}
    </div>
  );
}
