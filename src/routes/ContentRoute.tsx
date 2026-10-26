import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  LOADER_KEYS,
  LOADER_ORDER,
  CONTENT_KIND_KEYS,
  CONTENT_KIND_ORDER,
} from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon, type IconName } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { SegmentedControl } from '@/components/ui/SegmentedControl';
import { InstalledContentGroups } from '@/components/InstalledContentGroups';
import type { ContentKind, ContentProviderId } from '@/ipc/commands';
import { describeError } from '@/ipc/messages';
import {
  useContentProviders,
  useInstallContent,
  useInstalledContent,
  useInstances,
  useRemoveContent,
  useSearchContent,
  useVersions,
} from '@/ipc/hooks';

import pages from './pages.module.css';

const KIND_ICONS: Readonly<Record<ContentKind, IconName>> = {
  mod: 'package',
  resourcepack: 'layers',
  shader: 'sparkles',
  world: 'globe',
  datapack: 'database',
};

const RESULT_LIMIT = 20;

export function ContentRoute() {
  const { t, i18n } = useTranslation();
  const providers = useContentProviders();
  const versions = useVersions();
  const instances = useInstances();

  const [provider, setProvider] = useState<ContentProviderId>('modrinth');
  const [kind, setKind] = useState<ContentKind>('mod');
  const [query, setQuery] = useState('');
  const [submitted, setSubmitted] = useState('');
  const [gameVersion, setGameVersion] = useState('');
  const [loader, setLoader] = useState('');
  const [instanceId, setInstanceId] = useState('');
  const [error, setError] = useState<string | null>(null);

  const search = useSearchContent(
    submitted.trim().length === 0
      ? undefined
      : {
          provider,
          query: submitted.trim(),
          kind,
          ...(gameVersion === '' ? {} : { gameVersion }),
          ...(loader === '' ? {} : { loader }),
          limit: RESULT_LIMIT,
        },
  );
  const installed = useInstalledContent(instanceId === '' ? undefined : instanceId);
  const installContent = useInstallContent();
  const removeContent = useRemoveContent();

  const availableProviders = providers.data ?? [];
  const providerMissing = availableProviders.length === 0;
  const instanceRows = instances.data ?? [];
  const target = instanceRows.find((instance) => instance.id === instanceId);

  const kindOptions = useMemo(
    () =>
      CONTENT_KIND_ORDER.map((candidate) => ({
        value: candidate,
        label: t(CONTENT_KIND_KEYS[candidate]),
        icon: KIND_ICONS[candidate],
      })),
    [t],
  );

  const numberFormat = useMemo(() => new Intl.NumberFormat(i18n.language), [i18n.language]);

  function install(projectId: string): void {
    setError(null);
    if (instanceId === '') {
      setError(t('content.noInstances'));
      return;
    }
    installContent
      .mutateAsync({ instanceId, provider, projectId, kind })
      .catch((cause: unknown) => {
        setError(describeError(cause, t));
      });
  }

  function remove(projectId: string): void {
    setError(null);
    removeContent.mutateAsync({ instanceId, projectId }).catch((cause: unknown) => {
      setError(describeError(cause, t));
    });
  }

  return (
    <div className={pages.page}>
      <PageHeader title={t('content.heading')} description={t('content.description')} />

      <Card
        title={t('content.browseTitle')}
        description={t('content.browseBody')}
        icon="library"
        badge={
          search.data === undefined ? null : (
            <Badge tone="neutral">{t('content.results', { count: search.data.hits.length })}</Badge>
          )
        }
      >
        <div className={pages.wideBlock}>
          <div className={pages.toolbar}>
            {providerMissing ? null : (
              <div className={pages.field}>
                <label className={pages.fieldLabel} htmlFor="browse-provider">
                  {t('content.provider')}
                </label>
                <select
                  id="browse-provider"
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
              <label className={pages.fieldLabel} htmlFor="browse-version">
                {t('content.version')}
              </label>
              <select
                id="browse-version"
                className={pages.select}
                value={gameVersion}
                onChange={(event) => {
                  setGameVersion(event.target.value);
                }}
              >
                <option value="">{t('content.versionAny')}</option>
                {(versions.data?.versions ?? []).map((entry) => (
                  <option key={entry.id} value={entry.id}>
                    {entry.id}
                  </option>
                ))}
              </select>
            </div>
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="browse-loader">
                {t('content.loader')}
              </label>
              <select
                id="browse-loader"
                className={pages.select}
                value={loader}
                onChange={(event) => {
                  setLoader(event.target.value);
                }}
              >
                <option value="">{t('content.loaderAny')}</option>
                {LOADER_ORDER.map((candidate) => (
                  <option key={candidate} value={candidate}>
                    {t(LOADER_KEYS[candidate])}
                  </option>
                ))}
              </select>
            </div>
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="browse-instance">
                {t('content.target')}
              </label>
              <select
                id="browse-instance"
                className={pages.select}
                value={instanceId}
                onChange={(event) => {
                  setInstanceId(event.target.value);
                }}
              >
                <option value="">{t('content.targetNone')}</option>
                {instanceRows.map((instance) => (
                  <option key={instance.id} value={instance.id}>
                    {instance.name}
                  </option>
                ))}
              </select>
            </div>
          </div>

          <SegmentedControl
            name="content-kind"
            label={t('content.kind')}
            value={kind}
            options={kindOptions}
            onChange={setKind}
          />

          <form
            className={pages.inlineForm}
            onSubmit={(event) => {
              event.preventDefault();
              setSubmitted(query);
            }}
          >
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="browse-query">
                {t('content.query')}
              </label>
              <input
                id="browse-query"
                className={pages.input}
                value={query}
                onChange={(event) => {
                  setQuery(event.target.value);
                }}
              />
            </div>
            <Button
              type="submit"
              variant="primary"
              icon="compass"
              disabled={providerMissing || query.trim().length === 0}
            >
              {t('content.queryAction')}
            </Button>
          </form>

          {providerMissing ? <p className={pages.muted}>{t('content.noProviders')}</p> : null}

          {search.isError ? (
            <p className={pages.errorText} role="alert">
              {t('content.failed')}
            </p>
          ) : null}

          {search.isPending && submitted.length > 0 ? (
            <p className={pages.muted} role="status">
              <Icon name="loading" size={16} /> {t('content.searching')}
            </p>
          ) : null}

          {submitted.trim().length === 0 && search.data === undefined ? (
            <EmptyState
              icon="library"
              title={t('content.emptyTitle')}
              description={t('content.emptyBody')}
            />
          ) : null}

          {search.data !== undefined && search.data.hits.length === 0 ? (
            <p className={pages.muted}>{t('content.noResults')}</p>
          ) : null}

          <ul className={pages.list}>
            {search.data?.hits.map((project) => (
              <li key={`${project.provider}-${project.id}`} className={pages.row}>
                {project.iconUrl === undefined ? null : (
                  <img
                    className={pages.rowArt}
                    src={project.iconUrl}
                    alt=""
                    width={40}
                    height={40}
                  />
                )}
                <div className={pages.rowMain}>
                  <span className={pages.rowTitle}>{project.name}</span>
                  <span className={pages.rowMeta}>{project.summary}</span>
                  <span className={pages.rowMeta}>
                    {project.author} ·{' '}
                    {t('content.downloads', { total: numberFormat.format(project.downloads) })}
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
                  {t('content.install')}
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

      <Card
        title={
          target === undefined
            ? t('content.installedTitle')
            : t('content.installedTitleFor', { name: target.name })
        }
        description={t('content.installedBody')}
        icon="package"
      >
        {instanceId === '' ? (
          <EmptyState
            icon="instances"
            title={t('content.installedPickTitle')}
            description={t('content.installedPickBody')}
          />
        ) : (
          <InstalledContentGroups
            items={installed.data ?? []}
            removeLabel={t('content.remove')}
            disabled={removeContent.isPending}
            onRemove={remove}
          />
        )}
      </Card>
    </div>
  );
}
