import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate, useParams } from 'react-router-dom';

import { LOADER_KEYS, LOADER_ORDER } from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Dialog } from '@/components/ui/Dialog';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon } from '@/components/ui/Icon';
import { InstanceSettingsDialog } from '@/components/InstanceSettingsDialog';
import { PageHeader } from '@/components/ui/PageHeader';
import type { InstanceSummary, LaunchSession, LoaderKind } from '@/ipc/commands';
import { describeError } from '@/ipc/messages';
import {
  useAccounts,
  useCreateInstance,
  useDeleteInstance,
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

/** Recently launched first, then most recently created — the order a player scans for. */
function byRecency(left: InstanceSummary, right: InstanceSummary): number {
  const launched = (right.lastLaunchedMs ?? 0) - (left.lastLaunchedMs ?? 0);
  return launched === 0 ? right.createdAtMs - left.createdAtMs : launched;
}

export function InstancesRoute() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();

  const instances = useInstances();
  const sessions = useLaunchSessions();
  const accounts = useAccounts();
  const installInstance = useInstallInstance();
  const launchInstance = useLaunchInstance();
  const stopInstance = useStopInstance();
  const deleteInstance = useDeleteInstance();

  const [createOpen, setCreateOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [deleteId, setDeleteId] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const rows = useMemo(() => [...(instances.data ?? [])].sort(byRecency), [instances.data]);
  const selected = rows.find((row) => row.id === id);
  const selectedAccount = accounts.data?.find((account) => account.selected);
  const deleteTarget = rows.find((row) => row.id === deleteId);

  const allSessions = sessions.data ?? [];
  const runningSession =
    selected === undefined
      ? undefined
      : allSessions.find(
          (session) => session.instanceId === selected.id && session.state === 'running',
        );
  const latestSession = allSessions
    .filter((session) => session.instanceId === selected?.id)
    .reduce<LaunchSession | undefined>(
      (newest, session) =>
        newest === undefined || session.startedAtMs > newest.startedAtMs ? session : newest,
      undefined,
    );

  // A stale or missing path segment never leaves the pane blank.
  useEffect(() => {
    if (selected === undefined && rows.length > 0) {
      const first = rows[0];
      if (first !== undefined) {
        void navigate(`/instances/${encodeURIComponent(first.id)}`, { replace: true });
      }
    }
  }, [selected, rows, navigate]);

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

      {instances.isPending && rows.length === 0 ? (
        <p className={pages.muted} role="status">
          <Icon name="loading" size={16} /> {t('instances.loading')}
        </p>
      ) : null}

      {rows.length === 0 && !instances.isPending ? (
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

      {rows.length === 0 ? null : (
        <div className={pages.workspace}>
          <Card>
            <ul className={pages.picker}>
              {rows.map((instance) => {
                const isRunning = allSessions.some(
                  (session) => session.instanceId === instance.id && session.state === 'running',
                );
                return (
                  <li key={instance.id}>
                    <button
                      type="button"
                      onClick={() => {
                        void navigate(`/instances/${encodeURIComponent(instance.id)}`);
                      }}
                      className={
                        instance.id === selected?.id
                          ? `${pages.pickerItem} ${pages.pickerItemActive}`
                          : pages.pickerItem
                      }
                    >
                      <span className={pages.pickerGlyph} aria-hidden="true">
                        <Icon name={instance.loader === 'vanilla' ? 'game' : 'layers'} size={18} />
                      </span>
                      <span className={pages.pickerMain}>
                        <span className={pages.pickerName}>{instance.name}</span>
                        <span className={pages.pickerMeta}>
                          {instance.gameVersion} · {t(LOADER_KEYS[instance.loader])}
                        </span>
                      </span>
                      {isRunning ? (
                        <Badge tone="success" icon="play">
                          {t('instances.running')}
                        </Badge>
                      ) : instance.installed ? null : (
                        <Badge tone="warning">{t('instances.notInstalled')}</Badge>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          </Card>

          {selected === undefined ? null : (
            <Card>
              <div className={pages.detail}>
                <header className={pages.detailHead}>
                  <span className={pages.detailGlyph} aria-hidden="true">
                    <Icon
                      name={selected.loader === 'vanilla' ? 'game' : 'layers'}
                      size={26}
                      strokeWidth={1.5}
                    />
                  </span>
                  <div className={pages.detailHeading}>
                    <h2 className={pages.detailTitle}>{selected.name}</h2>
                    <p className={pages.detailMeta}>
                      {selected.gameVersion} · {t(LOADER_KEYS[selected.loader])}
                      {selected.loaderVersion === undefined ? '' : ` ${selected.loaderVersion}`}
                    </p>
                  </div>
                  {runningSession === undefined ? null : (
                    <Badge tone="success" icon="play">
                      {t('instances.running')}
                    </Badge>
                  )}
                </header>

                <div className={pages.detailActions}>
                  {!selected.installed ? (
                    <Button
                      className={pages.launchButton}
                      variant="primary"
                      icon="downloads"
                      disabled={installInstance.isPending}
                      onClick={() => {
                        void run(installInstance.mutateAsync(selected.id)).then((ok) => {
                          if (ok) {
                            setNotice(t('instances.installQueued'));
                          }
                        });
                      }}
                    >
                      {t('instances.install')}
                    </Button>
                  ) : runningSession !== undefined ? (
                    <Button
                      className={pages.launchButton}
                      variant="danger"
                      icon="close"
                      disabled={stopInstance.isPending}
                      onClick={() => {
                        void run(stopInstance.mutateAsync(runningSession.id));
                      }}
                    >
                      {t('instances.stop')}
                    </Button>
                  ) : (
                    <Button
                      className={pages.launchButton}
                      variant="primary"
                      icon="play"
                      disabled={launchInstance.isPending || selectedAccount === undefined}
                      title={
                        selectedAccount === undefined ? t('instances.needsAccount') : undefined
                      }
                      onClick={() => {
                        void run(launchInstance.mutateAsync({ id: selected.id }));
                      }}
                    >
                      {t('instances.launch')}
                    </Button>
                  )}
                  <Button
                    variant="secondary"
                    icon="settings"
                    onClick={() => {
                      setSettingsOpen(true);
                    }}
                  >
                    {t('instances.settingsTitle')}
                  </Button>
                </div>

                {selected.installed && selectedAccount === undefined ? (
                  <p className={pages.caption}>
                    {t('instances.noAccount')}{' '}
                    <ButtonLink to="/accounts" variant="ghost" size="sm" iconRight="arrowRight">
                      {t('instances.addAccount')}
                    </ButtonLink>
                  </p>
                ) : selected.installed ? (
                  <p className={pages.caption}>
                    {t('instances.launchAs', { name: selectedAccount?.name ?? '' })}
                  </p>
                ) : null}

                <dl className={pages.detailFacts}>
                  <dt>{t('instances.lastLaunched')}</dt>
                  <dd>
                    {selected.lastLaunchedMs === undefined
                      ? t('instances.never')
                      : new Date(selected.lastLaunchedMs).toLocaleString()}
                  </dd>
                  <dt>{t('instances.directory')}</dt>
                  <dd>{selected.directory}</dd>
                </dl>
              </div>
            </Card>
          )}
        </div>
      )}

      {selected === undefined ? null : (
        <InstanceSettingsDialog
          key={selected.id}
          instance={selected}
          sessionId={latestSession?.id}
          open={settingsOpen}
          onClose={() => {
            setSettingsOpen(false);
          }}
          onRequestDelete={() => {
            setSettingsOpen(false);
            setDeleteId(selected.id);
          }}
        />
      )}

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
              disabled={deleteInstance.isPending}
              onClick={() => {
                if (deleteTarget === undefined) {
                  return;
                }
                void run(deleteInstance.mutateAsync(deleteTarget.id)).then((ok) => {
                  if (ok) {
                    setDeleteId(null);
                    void navigate('/instances', { replace: true });
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
  const [feedback, setFeedback] = useState<string | null>(null);

  function close(): void {
    setJson('');
    setError(null);
    setFeedback(null);
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
              setFeedback(null);
              importInstance
                .mutateAsync(json)
                .then((summary) => {
                  setFeedback(t('instances.imported', { name: summary.name }));
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
        {feedback === null ? null : (
          <p className={pages.successText} role="status">
            {feedback}
          </p>
        )}
      </div>
    </Dialog>
  );
}
