import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';
import { InstanceEntryList, type EntryListItem } from '@/components/InstanceEntryList';
import { useInstanceFiles } from '@/ipc/hooks';

import styles from './InstanceSettingsDialog.module.css';
import pages from '@/routes/pages.module.css';

/**
 * A read-only browse of one instance game directory.
 *
 * Paths are the instance-relative ones the core handed out, joined with `/`, so the
 * breadcrumb is a string operation rather than a second round trip. Nothing here can
 * reach outside the instance: the core re-checks every path it is given.
 */
export function InstanceFilesPanel({ instanceId }: { readonly instanceId: string }) {
  const { t } = useTranslation();
  const [dir, setDir] = useState('');
  const listing = useInstanceFiles(instanceId, dir, true);

  const items: EntryListItem[] = (listing.data?.entries ?? []).map((entry) => ({ ...entry }));
  const shownPath = listing.data === undefined ? dir : listing.data.path;

  return (
    <>
      <p className={pages.muted}>{t('instances.filesBody')}</p>
      <div className={pages.inlineForm}>
        <Button
          size="sm"
          variant="ghost"
          icon="chevronRight"
          disabled={dir.length === 0}
          onClick={() => {
            setDir(dir.slice(0, Math.max(dir.lastIndexOf('/'), 0)));
          }}
        >
          {t('instances.filesUp')}
        </Button>
        <span className={styles.path}>
          {shownPath.length === 0 ? t('instances.filesRoot') : shownPath}
        </span>
      </div>
      <p className={pages.caption}>{t('instances.filesFolderNotice')}</p>

      {listing.isError ? (
        <p className={pages.errorText}>{t('instances.entriesUnavailable')}</p>
      ) : listing.isPending ? (
        <p className={pages.muted}>{t('common.loading')}</p>
      ) : (
        <>
          {listing.data?.truncated === true ? (
            <p className={pages.muted}>{t('instances.filesTruncated')}</p>
          ) : null}
          <InstanceEntryList
            instanceId={instanceId}
            items={items}
            open={{
              label: t('common.open'),
              onSelect: (item) => {
                setDir(item.path);
              },
            }}
            empty={
              <EmptyState
                icon="folder"
                title={t('instances.filesEmptyTitle')}
                description={t('instances.filesEmptyBody')}
              />
            }
          />
        </>
      )}
    </>
  );
}
