import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate, useParams } from 'react-router-dom';

import { CONTENT_KIND_KEYS, CONTENT_KIND_ORDER, LOADER_KEYS } from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Dialog } from '@/components/ui/Dialog';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { InstalledContentGroups } from '@/components/InstalledContentGroups';
import type { ContentKind, ContentProviderId } from '@/ipc/commands';
import { describeError } from '@/ipc/messages';
import {
  useAccounts,
  useContentProviders,
  useDeleteInstance,
  useExportInstance,
  useInstallContent,
  useInstallInstance,
  useInstance,
  useInstalledContent,
  useLaunchInstance,
  useLaunchLogs,
  useLaunchSessions,
  useRemoveContent,
  useSearchContent,
  useStopInstance,
  useUpdateInstance,
} from '@/ipc/hooks';

import pages from './pages.module.css';

export function InstanceDetailRoute() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();

  const instance = useInstance(id);
  const sessions = useLaunchSessions();
  const accounts = useAccounts();
  const updateInstance = useUpdateInstance();
  const deleteInstance = useDeleteInstance();
  const installInstance = useInstallInstance();
  const launchInstance = useLaunchInstance();
  const stopInstance = useStopInstance();
  const exportInstance = useExportInstance();

  const [nameDraft, setNameDraft] = useState<string | null>(null);
  const [javaPathDraft, setJavaPathDraft] = useState<string | null>(null);
  const [deleteOpen, setDeleteOpen] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const name = nameDraft ?? instance.data?.name ?? '';
  const javaPath = javaPathDraft ?? instance.data?.javaPath ?? '';

  const latestSession = useMemo(() => {
    const mine = (sessions.data ?? []).filter((session) => session.instanceId === id);
    return (
      mine.find((session) => session.state === 'running') ??
      [...mine].sort((left, right) => right.startedAtMs - left.startedAtMs)[0]
    );
  }, [sessions.data, id]);

  const selectedAccount = accounts.data?.find((account) => account.selected);

  function run(action: Promise<unknown>): Promise<boolean> {
    setError(null);
    setNotice(null);
    return action.then(
      () => true,
      (cause: unknown) => {
        setError(describeError(cause, t));
        return false;
      },
    );
  }

  if (instance.isError) {
    return (
      <div className={pages.page}>
        <PageHeader
          title={t('instances.detailTitle')}
          description={t('instances.loadError')}
          actions={
            <ButtonLink to="/instances" variant="secondary" icon="arrowRight">
              {t('instances.back')}
            </ButtonLink>
          }
        />
        <Card icon="warning" title={t('instances.notFound')}>
          <Button
            variant="secondary"
            icon="refresh"
            onClick={() => {
              void instance.refetch();
            }}
          >
            {t('error.retry')}
          </Button>
        </Card>
      </div>
    );
  }

  if (instance.data === undefined) {
    return (
      <div className={pages.page}>
        <p className={pages.muted} role="status">
          <Icon name="loading" size={16} /> {t('instances.loading')}
        </p>
      </div>
    );
  }

  const summary = instance.data;
  const running = latestSession?.state === 'running';

  return (
    <div className={pages.page}>
      <PageHeader
        eyebrow={
          <ButtonLink to="/instances" variant="ghost" size="sm" icon="instances">
            {t('instances.back')}
          </ButtonLink>
        }
        title={summary.name}
        description={`${summary.gameVersion} · ${t(LOADER_KEYS[summary.loader])} · ${summary.versionId}`}
        badge={
          <Badge tone={summary.installed ? 'success' : 'warning'}>
            {summary.installed ? t('instances.installed') : t('instances.notInstalled')}
          </Badge>
        }
        actions={
          <>
            {summary.installed ? (
              running ? (
                <Button
                  variant="danger"
                  icon="close"
                  disabled={stopInstance.isPending}
                  onClick={() => {
                    if (latestSession === undefined) {
                      return;
                    }
                    void run(stopInstance.mutateAsync(latestSession.id));
                  }}
                >
                  {t('instances.stop')}
                </Button>
              ) : (
                <Button
                  variant="primary"
                  icon="play"
                  disabled={launchInstance.isPending}
                  title={selectedAccount === undefined ? t('instances.needsAccount') : undefined}
                  onClick={() => {
                    void run(launchInstance.mutateAsync({ id: summary.id }));
                  }}
                >
                  {t('instances.launch')}
                </Button>
              )
            ) : (
              <Button
                variant="primary"
                icon="downloads"
                disabled={installInstance.isPending}
                onClick={() => {
                  void run(installInstance.mutateAsync(summary.id)).then((ok) => {
                    if (ok) {
                      setNotice(t('instances.installQueued'));
                    }
                  });
                }}
              >
                {t('instances.install')}
              </Button>
            )}
            <Button
              variant="secondary"
              icon="package"
              disabled={exportInstance.isPending}
              onClick={() => {
                void run(exportInstance.mutateAsync(summary.id)).then((ok) => {
                  if (ok) {
                    setNotice(t('instances.exported', { id: summary.id }));
                  }
                });
              }}
            >
              {t('instances.export')}
            </Button>
            <Button
              variant="ghost"
              icon="trash"
              onClick={() => {
                setDeleteOpen(true);
              }}
            >
              {t('instances.delete')}
            </Button>
          </>
        }
      />

      {selectedAccount === undefined ? (
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

      <div className={pages.grid}>
        <Card title={t('instances.overview')} icon="info">
          <dl className={pages.facts}>
            <dt>{t('instances.gameVersion')}</dt>
            <dd>{summary.gameVersion}</dd>
            <dt>{t('instances.loader')}</dt>
            <dd>
              {t(LOADER_KEYS[summary.loader])}
              {summary.loaderVersion === undefined ? '' : ` ${summary.loaderVersion}`}
            </dd>
            <dt>{t('instances.versionId')}</dt>
            <dd>{summary.versionId}</dd>
            <dt>{t('instances.directory')}</dt>
            <dd>{summary.directory}</dd>
            <dt>{t('instances.lastLaunched')}</dt>
            <dd>
              {summary.lastLaunchedMs === undefined
                ? t('instances.never')
                : new Date(summary.lastLaunchedMs).toLocaleString()}
            </dd>
          </dl>
        </Card>

        <Card title={t('instances.settingsTitle')} icon="settings">
          <div className={pages.settingBlock}>
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
                  setNameDraft(event.target.value);
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
                  setJavaPathDraft(event.target.value);
                }}
              />
            </div>
            <div>
              <Button
                variant="secondary"
                icon="check"
                disabled={updateInstance.isPending || name.trim().length === 0}
                onClick={() => {
                  void run(
                    updateInstance.mutateAsync({
                      id: summary.id,
                      update: { name: name.trim(), javaPath: javaPath.trim() },
                    }),
                  ).then((ok) => {
                    if (ok) {
                      setNameDraft(null);
                      setJavaPathDraft(null);
                      setNotice(t('instances.saved'));
                    }
                  });
                }}
              >
                {t('common.save')}
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <ContentSection
        instanceId={summary.id}
        gameVersion={summary.gameVersion}
        loader={summary.loader}
      />

      <Card
        title={t('instances.logs')}
        description={t('instances.logsBody')}
        icon="terminal"
        badge={
          latestSession === undefined ? undefined : (
            <Badge tone={running ? 'success' : 'neutral'}>
              {t(`instances.sessionState.${latestSession.state}`)}
            </Badge>
          )
        }
      >
        <LaunchLogPanel sessionId={latestSession?.id} />
      </Card>

      <Dialog
        open={deleteOpen}
        title={t('instances.deleteTitle')}
        description={t('instances.deleteBody', { name: summary.name })}
        closeLabel={t('common.cancel')}
        onClose={() => {
          setDeleteOpen(false);
        }}
        footer={
          <>
            <Button
              size="sm"
              variant="secondary"
              onClick={() => {
                setDeleteOpen(false);
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
                void run(deleteInstance.mutateAsync(summary.id)).then((ok) => {
                  if (ok) {
                    setDeleteOpen(false);
                    void navigate('/instances');
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

interface ContentSectionProps {
  readonly instanceId: string;
  readonly gameVersion: string;
  readonly loader: string;
}

function ContentSection({ instanceId, gameVersion, loader }: ContentSectionProps) {
  const { t, i18n } = useTranslation();
  const providers = useContentProviders();
  const installed = useInstalledContent(instanceId);
  const installContent = useInstallContent();
  const removeContent = useRemoveContent();

  const [provider, setProvider] = useState<ContentProviderId>('modrinth');
  const [kind, setKind] = useState<ContentKind>('mod');
  const [query, setQuery] = useState('');
  const [submitted, setSubmitted] = useState('');
  const [error, setError] = useState<string | null>(null);

  const search = useSearchContent(
    submitted.trim().length === 0
      ? undefined
      : {
          provider,
          query: submitted.trim(),
          kind,
          gameVersion,
          ...(loader === 'vanilla' ? {} : { loader }),
          limit: 20,
        },
  );

  const availableProviders = providers.data ?? [];
  const providerMissing = availableProviders.length === 0;
  const numberFormat = useMemo(() => new Intl.NumberFormat(i18n.language), [i18n.language]);

  function install(projectId: string): void {
    setError(null);
    installContent
      .mutateAsync({ instanceId, provider, projectId, kind })
      .catch((cause: unknown) => {
        setError(describeError(cause, t));
      });
  }

  return (
    <>
      <Card
        title={t('instances.contentTitle')}
        description={t('instances.contentBody')}
        icon="package"
        badge={
          <Badge tone="neutral">
            {t('instances.contentInstalled', { count: installed.data?.length ?? 0 })}
          </Badge>
        }
      >
        <div className={pages.wideBlock}>
          <div className={pages.toolbar}>
            {providerMissing ? null : (
              <div className={pages.field}>
                <label className={pages.fieldLabel} htmlFor="content-provider">
                  {t('instances.contentProvider')}
                </label>
                <select
                  id="content-provider"
                  className={pages.select}
                  value={provider}
                  onChange={(event) => {
                    setProvider(event.target.value as ContentProviderId);
                  }}
                >
                  {availableProviders.map((id) => (
                    <option key={id} value={id}>
                      {t(`contentProvider.${id}`)}
                    </option>
                  ))}
                </select>
              </div>
            )}
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="content-kind">
                {t('instances.contentKind')}
              </label>
              <select
                id="content-kind"
                className={pages.select}
                value={kind}
                onChange={(event) => {
                  setKind(event.target.value as ContentKind);
                }}
              >
                {CONTENT_KIND_ORDER.map((candidate) => (
                  <option key={candidate} value={candidate}>
                    {t(CONTENT_KIND_KEYS[candidate])}
                  </option>
                ))}
              </select>
            </div>
            <form
              className={pages.inlineForm}
              onSubmit={(event) => {
                event.preventDefault();
                setSubmitted(query);
              }}
            >
              <div className={pages.field}>
                <label className={pages.fieldLabel} htmlFor="content-query">
                  {t('instances.contentSearch')}
                </label>
                <input
                  id="content-query"
                  className={pages.input}
                  value={query}
                  onChange={(event) => {
                    setQuery(event.target.value);
                  }}
                />
              </div>
              <Button
                type="submit"
                variant="secondary"
                icon="compass"
                disabled={providerMissing || query.trim().length === 0}
              >
                {t('instances.contentSearchAction')}
              </Button>
            </form>
          </div>

          {providerMissing ? (
            <p className={pages.muted}>{t('instances.contentNoProviders')}</p>
          ) : null}

          {search.isError ? (
            <p className={pages.errorText} role="alert">
              {t('instances.contentSearchFailed')}
            </p>
          ) : null}

          {search.isPending && submitted.length > 0 ? (
            <p className={pages.muted} role="status">
              <Icon name="loading" size={16} /> {t('instances.contentSearching')}
            </p>
          ) : null}

          {search.data !== undefined && search.data.hits.length === 0 ? (
            <p className={pages.muted}>{t('instances.contentNoResults')}</p>
          ) : null}

          <ul className={pages.list}>
            {search.data?.hits.map((project) => (
              <li key={`${project.provider}-${project.id}`} className={pages.row}>
                <div className={pages.rowMain}>
                  <span className={pages.rowTitle}>{project.name}</span>
                  <span className={pages.rowMeta}>{project.summary}</span>
                  <span className={pages.rowMeta}>
                    {project.author} ·{' '}
                    {t('instances.contentDownloads', {
                      total: numberFormat.format(project.downloads),
                    })}
                  </span>
                </div>
                <Badge tone="neutral">{t(CONTENT_KIND_KEYS[project.kind])}</Badge>
                <Button
                  size="sm"
                  variant="primary"
                  icon="downloads"
                  disabled={installContent.isPending}
                  onClick={() => {
                    install(project.id);
                  }}
                >
                  {t('instances.contentInstall')}
                </Button>
              </li>
            ))}
          </ul>

          {error === null ? null : (
            <p className={pages.errorText} role="alert">
              {error}
            </p>
          )}
        </div>
      </Card>

      <Card title={t('instances.installedTitle')} icon="library">
        <InstalledContentGroups
          items={installed.data ?? []}
          removeLabel={t('instances.contentRemove')}
          disabled={removeContent.isPending}
          emptyTitle={t('instances.installedEmptyTitle')}
          emptyBody={t('instances.installedEmptyBody')}
          onRemove={(projectId) => {
            setError(null);
            removeContent.mutateAsync({ instanceId, projectId }).catch((cause: unknown) => {
              setError(describeError(cause, t));
            });
          }}
        />
      </Card>
    </>
  );
}

interface LaunchLogPanelProps {
  readonly sessionId: string | undefined;
}

function LaunchLogPanel({ sessionId }: LaunchLogPanelProps) {
  const { t } = useTranslation();
  const logs = useLaunchLogs(sessionId, sessionId !== undefined);

  if (sessionId === undefined) {
    return <p className={pages.muted}>{t('instances.noLogs')}</p>;
  }
  if (logs.isError) {
    return <p className={pages.errorText}>{t('instances.logsFailed')}</p>;
  }
  const lines = logs.data ?? [];
  if (lines.length === 0) {
    return <p className={pages.muted}>{t('instances.logsEmpty')}</p>;
  }
  return <pre className={`${pages.codeBox} ${pages.scrollList}`}>{lines.join('\n')}</pre>;
}
