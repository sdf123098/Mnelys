import { useState, type ReactNode } from 'react';
import type { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';

import { formatBytes, formatDateTime } from '@/app/format';
import { Button } from '@/components/ui/Button';
import { Icon, type IconName } from '@/components/ui/Icon';
import { IconButton } from '@/components/ui/IconButton';
import { useDeleteInstanceEntry, useRevealInstanceEntry } from '@/ipc/hooks';
import { describeError } from '@/ipc/messages';

import styles from './InstanceSettingsDialog.module.css';
import pages from '@/routes/pages.module.css';

/** One row of a directory-shaped list: worlds, schematics, log files or plain files. */
export interface EntryListItem {
  readonly name: string;
  /** Instance-relative path, which is also what the core deletes or reveals. */
  readonly path: string;
  readonly isDir: boolean;
  /** Zero for a directory in the file browser; a world carries the size its walk measured. */
  readonly sizeBytes: number;
  readonly modifiedMs: number | null;
  readonly icon?: IconName;
  /** True when the size walk stopped early, which makes the number a lower bound. */
  readonly sizeIsPartial?: boolean;
}

/** Opens an entry in place of deleting it: a folder in the browser, a file in the viewer. */
export interface EntryOpenAction {
  readonly label: string;
  readonly onSelect: (item: EntryListItem) => void;
}

interface InstanceEntryListProps {
  readonly instanceId: string;
  readonly items: readonly EntryListItem[];
  readonly empty: ReactNode;
  readonly open: EntryOpenAction | undefined;
}

/**
 * The list every instance filesystem panel shares.
 *
 * Deletion asks for confirmation inside the row rather than in a second dialog, because
 * the settings dialog is already modal and stacking a confirmation on top of it would
 * bury the entry the user is looking at.
 */
export function InstanceEntryList({ instanceId, items, empty, open }: InstanceEntryListProps) {
  const { t, i18n } = useTranslation();
  const deleteEntry = useDeleteInstanceEntry();
  const revealEntry = useRevealInstanceEntry();
  const [confirmingPath, setConfirmingPath] = useState<string | null>(null);
  const [failure, setFailure] = useState<string | null>(null);

  if (items.length === 0) {
    return <>{empty}</>;
  }

  return (
    <>
      {failure === null ? null : (
        <p className={pages.errorText} role="alert">
          {failure}
        </p>
      )}
      <ul className={`${pages.list} ${styles.entries}`}>
        {items.map((item) => (
          <EntryRow
            key={item.path}
            item={item}
            meta={entryMeta(item, t, i18n.language)}
            confirming={confirmingPath === item.path}
            busy={deleteEntry.isPending === true && deleteEntry.variables?.path === item.path}
            revealing={revealEntry.isPending === true && revealEntry.variables?.path === item.path}
            open={open}
            confirmBody={entryConfirmBody(item, t)}
            onConfirmChange={(next) => {
              setFailure(null);
              setConfirmingPath(next === true ? item.path : null);
            }}
            onReveal={() => {
              setFailure(null);
              revealEntry.mutate(
                { instanceId, path: item.path },
                { onError: (cause: unknown) => setFailure(describeError(cause, t)) },
              );
            }}
            onOpen={() => {
              if (open !== undefined) {
                open.onSelect(item);
              }
            }}
            onDelete={() => {
              setFailure(null);
              deleteEntry.mutate(
                { instanceId, path: item.path },
                {
                  onSuccess: () => {
                    setConfirmingPath(null);
                  },
                  onError: (cause: unknown) => setFailure(describeError(cause, t)),
                },
              );
            }}
          />
        ))}
      </ul>
    </>
  );
}

interface EntryRowProps {
  readonly item: EntryListItem;
  readonly meta: string;
  readonly confirmBody: string;
  readonly confirming: boolean;
  readonly busy: boolean;
  readonly revealing: boolean;
  readonly open: EntryOpenAction | undefined;
  readonly onConfirmChange: (confirming: boolean) => void;
  readonly onReveal: () => void;
  readonly onOpen: () => void;
  readonly onDelete: () => void;
}

function EntryRow({
  item,
  meta,
  confirmBody,
  confirming,
  busy,
  revealing,
  open,
  onConfirmChange,
  onReveal,
  onOpen,
  onDelete,
}: EntryRowProps) {
  const { t } = useTranslation();

  return (
    <li className={`${pages.row} ${styles.row}`}>
      <span className={styles.rowIcon}>
        <Icon name={item.icon ?? (item.isDir ? 'folder' : 'file')} size={16} />
      </span>
      <div className={`${pages.rowMain} ${styles.rowMain}`}>
        <span className={pages.rowTitle}>{item.name}</span>
        <span className={pages.rowMeta}>{confirming ? confirmBody : meta}</span>
      </div>
      <div className={pages.rowActions}>
        {confirming ? (
          <>
            <Button size="sm" variant="danger" icon="trash" disabled={busy} onClick={onDelete}>
              {t('instances.delete')}
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={busy}
              onClick={() => {
                onConfirmChange(false);
              }}
            >
              {t('common.cancel')}
            </Button>
          </>
        ) : (
          <>
            {open === undefined ? null : (
              <Button size="sm" variant="ghost" icon="chevronRight" onClick={onOpen}>
                {open.label}
              </Button>
            )}
            {/* Icon only, because this label is long in every catalog and it would
                otherwise push the size and date onto a second line. */}
            <IconButton
              icon="folder"
              label={t('instances.entryReveal')}
              disabled={revealing}
              onClick={onReveal}
            />
            <Button
              size="sm"
              variant="ghost"
              icon="trash"
              onClick={() => {
                onConfirmChange(true);
              }}
            >
              {t('instances.delete')}
            </Button>
          </>
        )}
      </div>
    </li>
  );
}

function entryMeta(item: EntryListItem, t: TFunction, locale: string): string {
  const modified =
    item.modifiedMs === null ? t('instances.timeUnknown') : formatDateTime(locale, item.modifiedMs);
  if (item.sizeBytes === 0) {
    return modified;
  }
  const partial = item.sizeIsPartial === true ? ` ${t('instances.sizePartial')}` : '';
  return `${formatBytes(item.sizeBytes)}${partial} · ${modified}`;
}

function entryConfirmBody(item: EntryListItem, t: TFunction): string {
  return item.isDir
    ? t('instances.entryConfirmFolderBody', { name: item.name })
    : t('instances.entryConfirmBody', { name: item.name });
}
