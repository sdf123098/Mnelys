import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { formatDateTime } from '@/app/format';
import { CONTENT_KIND_ICONS, CONTENT_KIND_KEYS, CONTENT_KIND_ORDER } from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Dialog } from '@/components/ui/Dialog';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon, type IconName } from '@/components/ui/Icon';
import { InstalledContentGroups } from '@/components/InstalledContentGroups';
import { InstanceDiagnosisPanel } from '@/components/InstanceDiagnosisPanel';
import { InstanceEntryList, type EntryListItem } from '@/components/InstanceEntryList';
import { InstanceFilesPanel } from '@/components/InstanceFilesPanel';
import { InstanceLogsPanel } from '@/components/InstanceLogsPanel';
import { SegmentedControl } from '@/components/ui/SegmentedControl';
import type { ContentKind, InstanceSummary } from '@/ipc/commands';
import { describeError } from '@/ipc/messages';
import {
  useExportInstance,
  useInstalledContent,
  useInstanceSchematics,
  useInstanceWorlds,
  useRemoveContent,
  useUpdateInstance,
} from '@/ipc/hooks';

import styles from './InstanceSettingsDialog.module.css';
import pages from '@/routes/pages.module.css';

type Category =
  'general' | 'content' | 'worlds' | 'schematics' | 'logs' | 'diagnosis' | 'files' | 'export';

const CATEGORIES: readonly { id: Category; icon: IconName }[] = [
  { id: 'general', icon: 'settings' },
  { id: 'content', icon: 'package' },
  { id: 'files', icon: 'folder' },
  { id: 'logs', icon: 'terminal' },
  { id: 'diagnosis', icon: 'diagnosis' },
  { id: 'schematics', icon: 'box' },
  { id: 'worlds', icon: 'globe' },
  { id: 'export', icon: 'downloads' },
];

interface InstanceSettingsDialogProps {
  readonly instance: InstanceSummary;
  /** Newest session of this instance, when one exists. Drives the live log pane. */
  readonly sessionId: string | undefined;
  readonly open: boolean;
  readonly onClose: () => void;
  readonly onRequestDelete: () => void;
}

/**
 * Everything that belongs to one instance folder, behind one category rail.
 *
 * Content, worlds, schematics, logs and files are separate surfaces because they are
 * deleted differently: removing a mod edits one file, removing a world removes a
 * directory, and removing a log removes evidence. Each pane therefore reads its own
 * listing instead of one merged tree.
 */
export function InstanceSettingsDialog({
  instance,
  sessionId,
  open,
  onClose,
  onRequestDelete,
}: InstanceSettingsDialogProps) {
  const { t } = useTranslation();
  const [category, setCategory] = useState<Category>('general');

  return (
    <Dialog
      open={open}
      size="wide"
      title={t('instances.settingsTitle')}
      description={instance.name}
      closeLabel={t('common.cancel')}
      onClose={onClose}
    >
      <div className={styles.grid}>
        <fieldset className={styles.nav}>
          <legend className={styles.legend}>{t('instances.settingsTitle')}</legend>
          {CATEGORIES.map((entry) => (
            <label
              key={entry.id}
              className={
                category === entry.id ? `${styles.option} ${styles.optionActive}` : styles.option
              }
            >
              <input
                className={styles.radio}
                type="radio"
                name="instance-settings-category"
                value={entry.id}
                checked={category === entry.id}
                onChange={() => {
                  setCategory(entry.id);
                }}
              />
              <Icon name={entry.icon} size={15} />
              {t(`instances.settingsCategory.${entry.id}`)}
            </label>
          ))}
        </fieldset>

        <div className={styles.panel}>
          {category === 'general' ? <GeneralPane instance={instance} /> : null}
          {category === 'content' ? <ContentPane instance={instance} /> : null}
          {category === 'files' ? <InstanceFilesPanel instanceId={instance.id} /> : null}
          {category === 'logs' ? (
            <InstanceLogsPanel instanceId={instance.id} sessionId={sessionId} />
          ) : null}
          {category === 'schematics' ? <SchematicsPane instanceId={instance.id} /> : null}
          {category === 'worlds' ? <WorldsPane instanceId={instance.id} /> : null}
          {category === 'diagnosis' ? <InstanceDiagnosisPanel instanceId={instance.id} /> : null}
          {category === 'export' ? (
            <ExportPane instance={instance} onRequestDelete={onRequestDelete} />
          ) : null}
        </div>
      </div>
    </Dialog>
  );
}

function GeneralPane({ instance }: { readonly instance: InstanceSummary }) {
  const { t, i18n } = useTranslation();
  const updateInstance = useUpdateInstance();

  const [name, setName] = useState(instance.name);
  const [javaPath, setJavaPath] = useState(instance.javaPath ?? '');
  const [feedback, setFeedback] = useState<string | null>(null);

  const dirty = name !== instance.name || javaPath !== (instance.javaPath ?? '');

  return (
    <>
      <div className={pages.field}>
        <label className={pages.fieldLabel} htmlFor="instance-edit-name">
          {t('instances.name')}
        </label>
        <input
          id="instance-edit-name"
          className={pages.input}
          value={name}
          maxLength={64}
          onChange={(event) => {
            setName(event.target.value);
            setFeedback(null);
          }}
        />
      </div>
      <div className={pages.field}>
        <label className={pages.fieldLabel} htmlFor="instance-edit-java">
          {t('instances.javaPath')}
        </label>
        <input
          id="instance-edit-java"
          className={pages.input}
          value={javaPath}
          spellCheck={false}
          placeholder={t('instances.javaAuto')}
          onChange={(event) => {
            setJavaPath(event.target.value);
            setFeedback(null);
          }}
        />
      </div>
      <div className={pages.inlineForm}>
        <Button
          size="sm"
          variant="primary"
          icon="check"
          disabled={!dirty || name.trim().length === 0 || updateInstance.isPending}
          onClick={() => {
            updateInstance.mutate(
              { id: instance.id, update: { name: name.trim(), javaPath: javaPath.trim() } },
              {
                onSuccess: () => {
                  setFeedback(t('instances.saved'));
                },
                onError: (cause: unknown) => {
                  setFeedback(describeError(cause, t));
                },
              },
            );
          }}
        >
          {t('common.save')}
        </Button>
      </div>
      {feedback === null ? null : <p className={pages.muted}>{feedback}</p>}

      <dl className={pages.facts}>
        <dt>{t('instances.versionId')}</dt>
        <dd>{instance.versionId}</dd>
        <dt>{t('instances.directory')}</dt>
        <dd>{instance.directory}</dd>
        <dt>{t('instances.created')}</dt>
        <dd>{formatDateTime(i18n.language, instance.createdAtMs)}</dd>
      </dl>
    </>
  );
}

type ContentFilter = ContentKind | 'all';

/**
 * Installed files of this instance.
 *
 * The tabs browse one resource type at a time, while a search term spans every type: the
 * grouping underneath still names each type, so nothing is ambiguous once the term is in.
 */
function ContentPane({ instance }: { readonly instance: InstanceSummary }) {
  const { t } = useTranslation();
  const installed = useInstalledContent(instance.id);
  const removeContent = useRemoveContent();
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<ContentFilter>('all');
  const [term, setTerm] = useState('');

  const needle = term.trim().toLocaleLowerCase();
  const searching = needle.length > 0;
  const tabIsOverridden = searching && filter !== 'all';

  const items = useMemo(() => {
    return (installed.data ?? []).filter((item) => {
      if (searching) {
        return (
          item.name.toLocaleLowerCase().includes(needle) ||
          item.filename.toLocaleLowerCase().includes(needle)
        );
      }
      return filter === 'all' || item.kind === filter;
    });
  }, [filter, installed.data, needle, searching]);

  const filtering = filter !== 'all' || searching;

  return (
    <>
      <p className={pages.muted}>{t('instances.contentIntro')}</p>
      <div className={pages.inlineForm}>
        <ButtonLink
          to={`/content?instance=${encodeURIComponent(instance.id)}`}
          size="sm"
          variant="secondary"
          icon="compass"
        >
          {t('instances.contentBrowse')}
        </ButtonLink>
        <Badge tone="neutral">
          {t('instances.contentInstalled', { count: installed.data?.length ?? 0 })}
        </Badge>
      </div>

      <SegmentedControl
        name="instance-content-kind"
        label={t('instances.contentFilter')}
        value={filter}
        options={[
          { value: 'all', label: t('instances.contentFilterAll'), icon: 'library' },
          ...CONTENT_KIND_ORDER.map((kind) => ({
            value: kind,
            label: t(CONTENT_KIND_KEYS[kind]),
            icon: CONTENT_KIND_ICONS[kind],
          })),
        ]}
        onChange={(value) => {
          setFilter(value);
        }}
      />
      <div className={`${pages.field} ${pages.fieldGrow}`}>
        <label className={pages.fieldLabel} htmlFor="instance-content-search">
          {t('instances.contentSearch')}
        </label>
        <input
          id="instance-content-search"
          className={pages.input}
          value={term}
          placeholder={t('instances.contentSearchPlaceholder')}
          onChange={(event) => {
            setTerm(event.target.value);
          }}
        />
      </div>
      {tabIsOverridden ? (
        <p className={pages.muted}>{t('instances.contentSearchAllKinds')}</p>
      ) : null}

      {error === null ? null : (
        <p className={pages.errorText} role="alert">
          {error}
        </p>
      )}
      {filtering && items.length === 0 ? (
        <p className={pages.muted}>{t('instances.contentNoMatches')}</p>
      ) : (
        <InstalledContentGroups
          items={items}
          removeLabel={t('instances.contentRemove')}
          disabled={removeContent.isPending}
          emptyTitle={t('instances.installedEmptyTitle')}
          emptyBody={t('instances.installedEmptyBody')}
          onRemove={(projectId) => {
            setError(null);
            removeContent
              .mutateAsync({ instanceId: instance.id, projectId })
              .catch((cause: unknown) => {
                setError(describeError(cause, t));
              });
          }}
        />
      )}
    </>
  );
}

function WorldsPane({ instanceId }: { readonly instanceId: string }) {
  const { t } = useTranslation();
  const worlds = useInstanceWorlds(instanceId, true);

  const items: EntryListItem[] = (worlds.data ?? []).map((world) => ({
    name: world.name,
    path: world.path,
    isDir: true,
    sizeBytes: world.sizeBytes,
    modifiedMs: world.modifiedMs,
    icon: 'globe',
    sizeIsPartial: !world.sizeComplete,
  }));

  return (
    <>
      <p className={pages.muted}>{t('instances.worldsBody')}</p>
      {worlds.isError ? (
        <p className={pages.errorText}>{t('instances.entriesUnavailable')}</p>
      ) : worlds.isPending ? (
        <p className={pages.muted}>{t('common.loading')}</p>
      ) : (
        <InstanceEntryList
          instanceId={instanceId}
          items={items}
          open={undefined}
          empty={
            <EmptyState
              icon="globe"
              title={t('instances.worldsEmptyTitle')}
              description={t('instances.worldsEmptyBody')}
            />
          }
        />
      )}
    </>
  );
}

function SchematicsPane({ instanceId }: { readonly instanceId: string }) {
  const { t } = useTranslation();
  const schematics = useInstanceSchematics(instanceId, true);

  const items: EntryListItem[] = (schematics.data ?? []).map((entry) => ({
    ...entry,
    icon: 'box',
  }));

  return (
    <>
      <p className={pages.muted}>{t('instances.schematicsBody')}</p>
      {schematics.isError ? (
        <p className={pages.errorText}>{t('instances.entriesUnavailable')}</p>
      ) : schematics.isPending ? (
        <p className={pages.muted}>{t('common.loading')}</p>
      ) : (
        <InstanceEntryList
          instanceId={instanceId}
          items={items}
          open={undefined}
          empty={
            <EmptyState
              icon="box"
              title={t('instances.schematicsEmptyTitle')}
              description={t('instances.schematicsEmptyBody')}
            />
          }
        />
      )}
    </>
  );
}

function ExportPane({
  instance,
  onRequestDelete,
}: {
  readonly instance: InstanceSummary;
  readonly onRequestDelete: () => void;
}) {
  const { t } = useTranslation();
  const exportInstance = useExportInstance();
  const [exportedPath, setExportedPath] = useState<string | null>(null);

  return (
    <>
      <p className={pages.muted}>{t('instances.exportBody')}</p>
      <div className={pages.inlineForm}>
        <Button
          size="sm"
          variant="secondary"
          icon="package"
          disabled={exportInstance.isPending}
          onClick={() => {
            setExportedPath(null);
            exportInstance.mutate(instance.id, {
              onSuccess: (result) => {
                setExportedPath(result.path);
              },
            });
          }}
        >
          {t('instances.export')}
        </Button>
        {exportInstance.isError ? (
          <span className={pages.errorText}>{t('instances.exportFailed')}</span>
        ) : null}
      </div>
      {exportedPath === null ? null : (
        <p className={pages.successText} role="status">
          {t('instances.exported', { path: exportedPath })}
        </p>
      )}

      <p className={pages.muted}>{t('instances.deleteBody', { name: instance.name })}</p>
      <div>
        <Button size="sm" variant="danger" icon="trash" onClick={onRequestDelete}>
          {t('instances.delete')}
        </Button>
      </div>
    </>
  );
}
