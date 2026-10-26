import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';

import { LOADER_KEYS, LOADER_ORDER } from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Dialog } from '@/components/ui/Dialog';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import type { LoaderKind } from '@/ipc/commands';
import { describeError } from '@/ipc/messages';
import {
  useAccounts,
  useCreateInstance,
  useDeleteInstance,
  useExportInstance,
  useImportInstance,
  useInstallInstance,
  useInstances,
  useLaunchInstance,
  useLaunchSessions,
  useLoaderVersions,
  useStopInstance,
  useVersions,
} from '@/ipc/hooks';

import pages from './pages.module.css';

function slugify(value: string): string {
  return value
    .normalize('NFKD')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .slice(0, 48)
    .replace(/-+$/, '');
}

export function InstancesRoute() {
  const { t } = useTranslation();
  const instances = useInstances();
  const sessions = useLaunchSessions();
  const accounts = useAccounts();
  const installInstance = useInstallInstance();
  const deleteInstance = useDeleteInstance();
  const launchInstance = useLaunchInstance();
  const stopInstance = useStopInstance();
  const exportInstance = useExportInstance();

  const [createOpen, setCreateOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const selectedAccount = accounts.data?.find((account) => account.selected);
  const deleteTarget = instances.data?.find((instance) => instance.id === deleteId);

  function fail(error: unknown): Promise<boolean> {
    setActionError(describeError(error, t));
    return Promise.resolve(false);
  }

  function run(action: Promise<unknown>): Promise<boolean> {
    setActionError(null);
    setNotice(null);
    return action.then(() => true, fail);
  }

  if (instances.isError) {
    return (
      <div className={pages.page}>
        <PageHeader title={t('instances.heading')} description={t('instances.description')} />
        <Card title={t('instances.loadError')} icon="warning">
          <Button
            variant="secondary"
            icon="refresh"
            onClick={() => {
              void instances.refetch();
            }}
          >
            {t('error.retry')}
          </Button>
        </Card>
      </div>
    );
  }

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('instances.heading')}
        description={t('instances.description')}
        actions={
          <>
            <Button
              variant="secondary"
              icon="file"
              onClick={() => {
                setImportOpen(true);
              }}
            >
              {t('instances.import')}
            </Button>
            <Button
              variant="primary"
              icon="plus"
              onClick={() => {
                setCreateOpen(true);
              }}
            >
              {t('instances.create')}
            </Button>
          </>
        }
      />

      {actionError === null ? null : (
        <p className={pages.errorText} role="alert">
          {actionError}
        </p>
      )}
      {notice === null ? null : (
        <p className={pages.successText} role="status">
          {notice}
        </p>
      )}

      {instances.data !== undefined && instances.data.length === 0 ? (
        <EmptyState
          icon="instances"
          title={t('instances.emptyTitle')}
          description={t('instances.emptyBody')}
          action={
            <Button
              variant="primary"
              icon="plus"
              onClick={() => {
                setCreateOpen(true);
              }}
            >
              {t('instances.create')}
            </Button>
          }
        />
      ) : null}

      {selectedAccount === undefined && (instances.data?.length ?? 0) > 0 ? (
        <div className={pages.notice}>
          <Icon name="info" size={16} />
          <p>
            {t('instances.noAccount')}{' '}
            <ButtonLink to="/accounts" variant="ghost" size="sm" iconRight="arrowRight">
              {t('instances.addAccount')}
            </ButtonLink>
          </p>
        </div>
      ) : null}

      <ul className={pages.list}>
        {instances.data?.map((instance) => {
          const session = sessions.data?.find(
            (candidate) => candidate.instanceId === instance.id && candidate.state === 'running',
          );
          return (
            <li key={instance.id} className={pages.row}>
              <span className={pages.rowMeta} aria-hidden="true">
                <Icon name={instance.loader === 'vanilla' ? 'game' : 'layers'} size={20} />
              </span>
              <div className={pages.rowMain}>
                <span className={pages.rowTitle}>{instance.name}</span>
                <span className={pages.rowMeta}>
                  {instance.gameVersion} · {t(LOADER_KEYS[instance.loader])}
                  {instance.loaderVersion === undefined ? '' : ` ${instance.loaderVersion}`} ·{' '}
                  {instance.versionId}
                </span>
              </div>
              {session === undefined ? null : (
                <Badge tone="success" icon="play">
                  {t('instances.running')}
                </Badge>
              )}
              <Badge tone={instance.installed ? 'success' : 'warning'}>
                {instance.installed ? t('instances.installed') : t('instances.notInstalled')}
              </Badge>
              <div className={pages.rowActions}>
                {instance.installed ? (
                  <Button
                    size="sm"
                    variant="primary"
                    icon="play"
                    disabled={session !== undefined || launchInstance.isPending}
                    title={
                      selectedAccount === undefined
                        ? t('instances.needsAccount')
                        : session === undefined
                          ? undefined
                          : t('instances.alreadyRunning')
                    }
                    onClick={() => {
                      void run(launchInstance.mutateAsync({ id: instance.id }));
                    }}
                  >
                    {t('instances.launch')}
                  </Button>
                ) : (
                  <Button
                    size="sm"
                    variant="primary"
                    icon="downloads"
                    disabled={installInstance.isPending}
                    onClick={() => {
                      void run(installInstance.mutateAsync(instance.id)).then((ok) => {
                        if (ok) {
                          setNotice(t('instances.installQueued'));
                        }
                      });
                    }}
                  >
                    {t('instances.install')}
                  </Button>
                )}
                {session === undefined ? null : (
                  <Button
                    size="sm"
                    variant="danger"
                    icon="close"
                    disabled={stopInstance.isPending}
                    onClick={() => {
                      void run(stopInstance.mutateAsync(session.id));
                    }}
                  >
                    {t('instances.stop')}
                  </Button>
                )}
                <ButtonLink
                  to={`/instances/${encodeURIComponent(instance.id)}`}
                  size="sm"
                  variant="secondary"
                  icon="settings"
                >
                  {t('instances.manage')}
                </ButtonLink>
                <Button
                  size="sm"
                  variant="ghost"
                  icon="package"
                  disabled={exportInstance.isPending}
                  onClick={() => {
                    void run(exportInstance.mutateAsync(instance.id)).then((ok) => {
                      if (ok) {
                        setNotice(t('instances.exported', { id: instance.id }));
                      }
                    });
                  }}
                >
                  {t('instances.export')}
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  icon="trash"
                  onClick={() => {
                    setDeleteId(instance.id);
                  }}
                >
                  {t('instances.delete')}
                </Button>
              </div>
            </li>
          );
        })}
      </ul>

      <CreateInstanceDialog
        open={createOpen}
        onClose={() => {
          setCreateOpen(false);
        }}
      />
      <ImportInstanceDialog
        open={importOpen}
        onClose={() => {
          setImportOpen(false);
        }}
      />

      <Dialog
        open={deleteId !== null}
        title={t('instances.deleteTitle')}
        description={t('instances.deleteBody', { name: deleteTarget?.name ?? deleteId ?? '' })}
        closeLabel={t('common.cancel')}
        onClose={() => {
          setDeleteId(null);
        }}
        footer={
          <>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setDeleteId(null);
              }}
            >
              {t('common.cancel')}
            </Button>
            <Button
              size="sm"
              variant="danger"
              icon="trash"
              onClick={() => {
                if (deleteTarget === undefined) {
                  return;
                }
                void run(deleteInstance.mutateAsync(deleteTarget.id)).then((ok) => {
                  if (ok) {
                    setDeleteId(null);
                  }
                });
              }}
            >
              {t('instances.delete')}
            </Button>
          </>
        }
      />
    </div>
  );
}

interface CreateInstanceDialogProps {
  readonly open: boolean;
  readonly onClose: () => void;
}

function CreateInstanceDialog({ open, onClose }: CreateInstanceDialogProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const createInstance = useCreateInstance();

  const [name, setName] = useState('');
  const [id, setId] = useState('');
  const [idTouched, setIdTouched] = useState(false);
  const [gameVersion, setGameVersion] = useState('');
  const [loader, setLoader] = useState<LoaderKind>('vanilla');
  const [loaderVersion, setLoaderVersion] = useState('');
  const [includeSnapshots, setIncludeSnapshots] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const versions = useVersions(open);
  const effectiveGameVersion =
    gameVersion.length > 0 ? gameVersion : open ? (versions.data?.latest.release ?? '') : '';
  const loaderVersions = useLoaderVersions(loader, effectiveGameVersion);

  const choices = useMemo(() => {
    const all = versions.data?.versions ?? [];
    return includeSnapshots ? all : all.filter((version) => version.type === 'release');
  }, [versions.data, includeSnapshots]);

  const loaderChoices = loaderVersions.data ?? [];
  const preferredLoaderVersion =
    loaderChoices.find((candidate) => candidate.stable) ?? loaderChoices[0];
  const effectiveLoaderVersion =
    loader === 'vanilla'
      ? ''
      : loaderVersion.length > 0 &&
          loaderChoices.some((candidate) => candidate.version === loaderVersion)
        ? loaderVersion
        : (preferredLoaderVersion?.version ?? '');

  function reset(): void {
    setName('');
    setId('');
    setIdTouched(false);
    setGameVersion('');
    setIncludeSnapshots(false);
    setLoader('vanilla');
    setLoaderVersion('');
    setError(null);
    createInstance.reset();
  }

  function close(): void {
    reset();
    onClose();
  }

  const needsLoaderVersion = loader !== 'vanilla';
  const ready =
    name.trim().length > 0 &&
    id.length > 0 &&
    effectiveGameVersion.length > 0 &&
    (!needsLoaderVersion || effectiveLoaderVersion.length > 0);

  return (
    <Dialog
      open={open}
      title={t('instances.createTitle')}
      description={t('instances.createBody')}
      closeLabel={t('common.cancel')}
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="secondary" onClick={close}>
            {t('common.cancel')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            icon="plus"
            disabled={!ready || createInstance.isPending}
            onClick={() => {
              setError(null);
              createInstance
                .mutateAsync({
                  id,
                  name: name.trim(),
                  gameVersion: effectiveGameVersion,
                  loader,
                  ...(effectiveLoaderVersion.length === 0
                    ? {}
                    : { loaderVersion: effectiveLoaderVersion }),
                })
                .then((summary) => {
                  close();
                  void navigate(`/instances/${encodeURIComponent(summary.id)}`);
                })
                .catch((cause: unknown) => {
                  setError(describeError(cause, t));
                });
            }}
          >
            {createInstance.isPending ? t('instances.creating') : t('instances.create')}
          </Button>
        </>
      }
    >
      <div className={pages.settingBlock}>
        <div className={pages.formGrid}>
          <div className={pages.field}>
            <label className={pages.fieldLabel} htmlFor="instance-name">
              {t('instances.name')}
            </label>
            <input
              id="instance-name"
              className={pages.input}
              value={name}
              maxLength={64}
              autoComplete="off"
              onChange={(event) => {
                const value = event.target.value;
                setName(value);
                if (!idTouched) {
                  setId(slugify(value));
                }
              }}
            />
          </div>
          <div className={pages.field}>
            <label className={pages.fieldLabel} htmlFor="instance-id">
              {t('instances.id')}
            </label>
            <input
              id="instance-id"
              className={pages.input}
              value={id}
              maxLength={48}
              spellCheck={false}
              autoComplete="off"
              onChange={(event) => {
                setIdTouched(true);
                setId(event.target.value.toLowerCase());
              }}
            />
            <span className={pages.muted}>{t('instances.idHint')}</span>
          </div>
        </div>

        <div className={pages.formGrid}>
          <div className={pages.field}>
            <label className={pages.fieldLabel} htmlFor="instance-version">
              {t('instances.gameVersion')}
            </label>
            <select
              id="instance-version"
              className={pages.select}
              value={effectiveGameVersion}
              disabled={versions.isPending}
              onChange={(event) => {
                setGameVersion(event.target.value);
                setLoaderVersion('');
              }}
            >
              {choices.map((version) => (
                <option key={version.id} value={version.id}>
                  {version.id}
                </option>
              ))}
            </select>
            <label className={pages.checkboxRow}>
              <input
                type="checkbox"
                checked={includeSnapshots}
                onChange={(event) => {
                  setIncludeSnapshots(event.target.checked);
                }}
              />
              {t('instances.includeSnapshots')}
            </label>
          </div>
          <div className={pages.field}>
            <label className={pages.fieldLabel} htmlFor="instance-loader">
              {t('instances.loader')}
            </label>
            <select
              id="instance-loader"
              className={pages.select}
              value={loader}
              onChange={(event) => {
                setLoader(event.target.value as LoaderKind);
                setLoaderVersion('');
              }}
            >
              {LOADER_ORDER.map((kind) => (
                <option key={kind} value={kind}>
                  {t(LOADER_KEYS[kind])}
                </option>
              ))}
            </select>
          </div>
          {needsLoaderVersion ? (
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="instance-loader-version">
                {t('instances.loaderVersion')}
              </label>
              <select
                id="instance-loader-version"
                className={pages.select}
                value={effectiveLoaderVersion}
                disabled={loaderVersions.isPending}
                onChange={(event) => {
                  setLoaderVersion(event.target.value);
                }}
              >
                {loaderChoices.map((candidate) => (
                  <option key={candidate.version} value={candidate.version}>
                    {candidate.version}
                    {candidate.stable ? '' : ` (${t('instances.experimental')})`}
                  </option>
                ))}
              </select>
              {loaderVersions.isError ? (
                <span className={pages.errorText}>{t('instances.loaderVersionsFailed')}</span>
              ) : null}
            </div>
          ) : null}
        </div>

        {versions.isError ? (
          <p className={pages.errorText} role="alert">
            {t('instances.versionsFailed')}
          </p>
        ) : null}
        {error === null ? null : (
          <p className={pages.errorText} role="alert">
            {error}
          </p>
        )}
      </div>
    </Dialog>
  );
}

interface ImportInstanceDialogProps {
  readonly open: boolean;
  readonly onClose: () => void;
}

function ImportInstanceDialog({ open, onClose }: ImportInstanceDialogProps) {
  const { t } = useTranslation();
  const importInstance = useImportInstance();
  const [json, setJson] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  function close(): void {
    setJson('');
    setError(null);
    setNotice(null);
    importInstance.reset();
    onClose();
  }

  return (
    <Dialog
      open={open}
      title={t('instances.importTitle')}
      description={t('instances.importBody')}
      closeLabel={t('common.cancel')}
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="secondary" onClick={close}>
            {t('common.cancel')}
          </Button>
          <Button
            size="sm"
            variant="primary"
            icon="file"
            disabled={json.trim().length === 0 || importInstance.isPending}
            onClick={() => {
              setError(null);
              setNotice(null);
              importInstance
                .mutateAsync(json)
                .then((summary) => {
                  setNotice(t('instances.imported', { name: summary.name }));
                  setJson('');
                })
                .catch((cause: unknown) => {
                  setError(describeError(cause, t));
                });
            }}
          >
            {t('instances.import')}
          </Button>
        </>
      }
    >
      <div className={pages.settingBlock}>
        <div className={pages.field}>
          <label className={pages.fieldLabel} htmlFor="instance-import-file">
            {t('instances.importFile')}
          </label>
          <input
            id="instance-import-file"
            className={pages.input}
            type="file"
            accept="application/json,.json"
            onChange={(event) => {
              const file = event.target.files?.[0];
              if (file === undefined) {
                return;
              }
              void file.text().then((text) => {
                setJson(text);
                setError(null);
              });
            }}
          />
        </div>
        <div className={pages.field}>
          <label className={pages.fieldLabel} htmlFor="instance-import-json">
            {t('instances.importJson')}
          </label>
          <textarea
            id="instance-import-json"
            className={pages.textarea}
            value={json}
            spellCheck={false}
            onChange={(event) => {
              setJson(event.target.value);
            }}
          />
        </div>
        {error === null ? null : (
          <p className={pages.errorText} role="alert">
            {error}
          </p>
        )}
        {notice === null ? null : (
          <p className={pages.successText} role="status">
            {notice}
          </p>
        )}
      </div>
    </Dialog>
  );
}
