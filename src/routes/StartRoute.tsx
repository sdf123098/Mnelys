import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import { LOADER_KEYS } from '@/app/wire-labels';
import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { describeError } from '@/ipc/messages';
import {
  useAccounts,
  useInstallInstance,
  useInstances,
  useLaunchInstance,
  useLaunchSessions,
  useStopInstance,
  useTasks,
} from '@/ipc/hooks';

import pages from './pages.module.css';

export function StartRoute() {
  const { t } = useTranslation();
  const accounts = useAccounts();
  const instances = useInstances();
  const sessions = useLaunchSessions();
  const tasks = useTasks(true);
  const launchInstance = useLaunchInstance();
  const installInstance = useInstallInstance();
  const stopInstance = useStopInstance();

  const selectedAccount = accounts.data?.find((account) => account.selected);

  const sortedInstances = useMemo(
    () =>
      [...(instances.data ?? [])].sort(
        (left, right) => (right.lastLaunchedMs ?? 0) - (left.lastLaunchedMs ?? 0),
      ),
    [instances.data],
  );

  const running = (sessions.data ?? []).filter((session) => session.state === 'running');
  const activeTasks = (tasks.data ?? []).filter(
    (task) => task.state === 'queued' || task.state === 'running',
  );

  const target =
    sortedInstances.find((instance) => instance.installed) ?? sortedInstances[0] ?? undefined;
  const targetRunning =
    target === undefined ? undefined : running.find((session) => session.instanceId === target.id);
  const readyTarget = target !== undefined && target.installed ? target : undefined;

  const error =
    launchInstance.error !== null
      ? describeError(launchInstance.error, t)
      : installInstance.error !== null
        ? describeError(installInstance.error, t)
        : stopInstance.error !== null
          ? describeError(stopInstance.error, t)
          : null;

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('start.heading')}
        description={t('start.description')}
        badge={
          <Badge tone={selectedAccount === undefined ? 'warning' : 'success'}>
            {selectedAccount === undefined ? t('start.cards.account.value') : selectedAccount.name}
          </Badge>
        }
      />

      {error === null ? null : (
        <p className={pages.errorText} role="alert">
          {error}
        </p>
      )}

      <section className={pages.hero}>
        <div className={pages.heroBody}>
          <h2>
            {readyTarget === undefined
              ? t('start.welcomeTitle')
              : t('start.readyTitle', { name: readyTarget.name })}
          </h2>
          <p>
            {readyTarget === undefined
              ? t('start.welcomeBody')
              : t('start.readyBody', {
                  version: readyTarget.gameVersion,
                  loader: t(LOADER_KEYS[readyTarget.loader]),
                })}
          </p>
          <div className={pages.heroActions}>
            {readyTarget !== undefined ? (
              targetRunning === undefined ? (
                <Button
                  variant="primary"
                  icon="play"
                  disabled={launchInstance.isPending || selectedAccount === undefined}
                  onClick={() => {
                    launchInstance.mutate({ id: readyTarget.id });
                  }}
                >
                  {t('start.launch')}
                </Button>
              ) : (
                <Button
                  variant="danger"
                  icon="close"
                  disabled={stopInstance.isPending}
                  onClick={() => {
                    stopInstance.mutate(targetRunning.id);
                  }}
                >
                  {t('instances.stop')}
                </Button>
              )
            ) : target === undefined ? null : (
              <Button
                variant="primary"
                icon="downloads"
                disabled={installInstance.isPending}
                onClick={() => {
                  installInstance.mutate(target.id);
                }}
              >
                {t('instances.install')}
              </Button>
            )}
            <ButtonLink to="/instances" variant="secondary" icon="instances">
              {t('start.openInstances')}
            </ButtonLink>
            {selectedAccount === undefined ? (
              <ButtonLink to="/accounts" variant="secondary" icon="accounts">
                {t('instances.addAccount')}
              </ButtonLink>
            ) : null}
          </div>
          <p className={pages.caption}>
            {selectedAccount === undefined
              ? t('start.launchNeedsAccount')
              : readyTarget !== undefined
                ? t('start.launchAccount', { name: selectedAccount.name })
                : t('start.launchDisabled')}
          </p>
        </div>
        <div className={pages.heroArt} aria-hidden="true">
          <span className={pages.heroGlyph}>
            <Icon name="game" size={64} strokeWidth={1.35} />
          </span>
        </div>
      </section>

      <div className={pages.grid}>
        <Card
          icon="accounts"
          title={t('start.cards.account.title')}
          description={t('start.cards.account.body')}
          badge={
            <Badge tone={selectedAccount === undefined ? 'neutral' : 'success'}>
              {selectedAccount === undefined
                ? t('start.cards.account.value')
                : selectedAccount.name}
            </Badge>
          }
          actions={
            <ButtonLink to="/accounts" variant="ghost" size="sm" iconRight="arrowRight">
              {t('common.open')}
            </ButtonLink>
          }
        />
        <Card
          icon="instances"
          title={t('start.cards.instances.title')}
          description={t('start.cards.instances.body')}
          badge={
            <Badge tone="neutral">
              {t('start.cards.instances.valueCount', { count: instances.data?.length ?? 0 })}
            </Badge>
          }
          actions={
            <ButtonLink to="/instances" variant="ghost" size="sm" iconRight="arrowRight">
              {t('common.open')}
            </ButtonLink>
          }
        />
        <Card
          icon="downloads"
          title={t('start.cards.downloads.title')}
          description={t('start.cards.downloads.body')}
          badge={
            <Badge tone={activeTasks.length > 0 ? 'info' : 'neutral'}>
              {activeTasks.length > 0
                ? t('start.cards.downloads.active', { count: activeTasks.length })
                : t('start.cards.downloads.value')}
            </Badge>
          }
          actions={
            <ButtonLink to="/downloads" variant="ghost" size="sm" iconRight="arrowRight">
              {t('common.open')}
            </ButtonLink>
          }
        />
      </div>

      {running.length === 0 ? null : (
        <Card title={t('start.runningTitle')} icon="play">
          <ul className={pages.list}>
            {running.map((session) => {
              const instance = instances.data?.find(
                (candidate) => candidate.id === session.instanceId,
              );
              return (
                <li key={session.id} className={pages.row}>
                  <span className={pages.rowMeta} aria-hidden="true">
                    <Icon name="game" size={18} />
                  </span>
                  <div className={pages.rowMain}>
                    <span className={pages.rowTitle}>{instance?.name ?? session.instanceId}</span>
                    <span className={pages.rowMeta}>
                      {t('start.runningMeta', {
                        pid: session.pid,
                        version: session.versionId,
                      })}
                    </span>
                  </div>
                  <Badge tone="success">{t('instances.running')}</Badge>
                  <Button
                    size="sm"
                    variant="danger"
                    icon="close"
                    disabled={stopInstance.isPending}
                    onClick={() => {
                      stopInstance.mutate(session.id);
                    }}
                  >
                    {t('instances.stop')}
                  </Button>
                </li>
              );
            })}
          </ul>
        </Card>
      )}

      <Card
        title={t('start.instancesTitle')}
        description={t('start.instancesBody')}
        icon="instances"
        actions={
          <ButtonLink to="/instances" variant="ghost" size="sm" iconRight="arrowRight">
            {t('common.open')}
          </ButtonLink>
        }
      >
        {sortedInstances.length === 0 ? (
          <p className={pages.muted}>{t('instances.emptyTitle')}</p>
        ) : (
          <ul className={pages.list}>
            {sortedInstances.slice(0, 5).map((instance) => (
              <li key={instance.id} className={pages.row}>
                <div className={pages.rowMain}>
                  <span className={pages.rowTitle}>{instance.name}</span>
                  <span className={pages.rowMeta}>
                    {instance.gameVersion} · {t(LOADER_KEYS[instance.loader])} ·{' '}
                    {instance.versionId}
                  </span>
                </div>
                <Badge tone={instance.installed ? 'success' : 'warning'}>
                  {instance.installed ? t('instances.installed') : t('instances.notInstalled')}
                </Badge>
                <ButtonLink
                  to={`/instances/${encodeURIComponent(instance.id)}`}
                  variant="secondary"
                  size="sm"
                  icon="settings"
                >
                  {t('instances.manage')}
                </ButtonLink>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
}
