import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useSearchParams } from 'react-router-dom';

import {
  CONTENT_KIND_ICONS,
  CONTENT_KIND_KEYS,
  CONTENT_KIND_ORDER,
  LOADER_KEYS,
  LOADER_ORDER,
} from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { SegmentedControl } from '@/components/ui/SegmentedControl';
import type { ContentKind, ContentProviderId } from '@/ipc/commands';
import { describeError } from '@/ipc/messages';
import {
  useContentProviders,
  useInstallContent,
  useInstances,
  useSearchContent,
  useVersions,
} from '@/ipc/hooks';

import pages from './pages.module.css';

const RESULT_LIMIT = 20;

export function ContentRoute() {
  const { t, i18n } = useTranslation();
  const [params] = useSearchParams();
  const providers = useContentProviders();
  const versions = useVersions();
  const instances = useInstances();

  const instanceRows = instances.data ?? [];
  const [provider, setProvider] = useState<ContentProviderId>('modrinth');
  const [kind, setKind] = useState<ContentKind>('mod');
  const [query, setQuery] = useState('');
  const [submitted, setSubmitted] = useState('');
  const [gameVersion, setGameVersion] = useState('');
  const [loader, setLoader] = useState('');
  const [instanceId, setInstanceId] = useState(params.get('instance') ?? '');
  const [error, setError] = useState<string | null>(null);
  const [failedIcons, setFailedIcons] = useState<ReadonlySet<string>>(new Set());

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
  const installContent = useInstallContent();

  const availableProviders = providers.data ?? [];
  const providerMissing = availableProviders.length === 0;
  const target = instanceRows.find((instance) => instance.id === instanceId);

  const kindOptions = useMemo(
    () =>
      CONTENT_KIND_ORDER.map((candidate) => ({
        value: candidate,
        label: t(CONTENT_KIND_KEYS[candidate]),
        icon: CONTENT_KIND_ICONS[candidate],
      })),
    [t],
  );

  const numberFormat = useMemo(() => new Intl.NumberFormat(i18n.language), [i18n.language]);

  /**
   * A target instance knows its own version and loader. Following them keeps a search
   * from returning files that cannot load in the instance being installed into.
   */
  function chooseInstance(nextId: string): void {
    setInstanceId(nextId);
    setError(null);
    const next = instanceRows.find((instance) => instance.id === nextId);
    if (next === undefined) {
      return;
    }
    setGameVersion(next.gameVersion);
    setLoader(next.loader === 'vanilla' ? '' : next.loader);
  }

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
            <div className={`${pages.field} ${pages.fieldGrow}`}>
              <label className={pages.fieldLabel} htmlFor="browse-query">
                {t('content.query')}
              </label>
              <input
                id="browse-query"
                className={pages.input}
                value={query}
                spellCheck={false}
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
                  chooseInstance(event.target.value);
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
            {target === undefined ? null : (
              <ButtonLink
                to={`/instances/${encodeURIComponent(target.id)}`}
                variant="ghost"
                size="sm"
                iconRight="arrowRight"
              >
                {t('content.installedLink')}
              </ButtonLink>
            )}
          </div>

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

          {error === null ? null : (
            <p className={pages.errorText} role="alert">
              {error}
            </p>
          )}

          {search.data === undefined || search.data.hits.length === 0 ? null : (
            <ul className={pages.results}>
              {search.data.hits.map((project) => (
                <li key={`${project.provider}-${project.id}`} className={pages.result}>
                  <div className={pages.resultHead}>
                    {project.iconUrl === undefined || failedIcons.has(project.id) ? (
                      <span className={pages.resultGlyph} aria-hidden="true">
                        <Icon name={CONTENT_KIND_ICONS[project.kind]} size={20} />
                      </span>
                    ) : (
                      <img
                        className={pages.resultArt}
                        src={project.iconUrl}
                        alt=""
                        width={44}
                        height={44}
                        onError={() => {
                          setFailedIcons((previous) => new Set(previous).add(project.id));
                        }}
                      />
                    )}
                    <div className={pages.resultMain}>
                      <span className={pages.resultName} title={project.name}>
                        {project.name}
                      </span>
                      <span className={pages.resultMeta}>{project.author}</span>
                    </div>
                  </div>
                  <p className={pages.resultSummary}>{project.summary}</p>
                  <div className={pages.resultFoot}>
                    <span className={pages.resultMeta}>
                      {t('content.downloads', { total: numberFormat.format(project.downloads) })}
                    </span>
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
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
      </Card>
    </div>
  );
}
