import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Dialog } from '@/components/ui/Dialog';
import { Icon, type IconName } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { SegmentedControl, type SegmentOption } from '@/components/ui/SegmentedControl';
import type { AccountKind, DeviceCodeChallenge } from '@/ipc/commands';
import {
  useAccounts,
  useAddOfflineAccount,
  useAddYggdrasilAccount,
  useBeginMicrosoftLogin,
  useCompleteMicrosoftLogin,
  useRefreshAccount,
  useRemoveAccount,
  useSelectAccount,
} from '@/ipc/hooks';

import pages from './pages.module.css';

type AddMethod = 'microsoft' | 'offline' | 'yggdrasil';

const KIND_ICONS: Readonly<Record<AccountKind, IconName>> = {
  microsoft: 'accounts',
  offline: 'game',
  yggdrasil: 'server',
};

const KIND_KEYS = {
  microsoft: 'accounts.kind.microsoft',
  offline: 'accounts.kind.offline',
  yggdrasil: 'accounts.kind.yggdrasil',
} as const satisfies Readonly<Record<AccountKind, string>>;

export function AccountsRoute() {
  const { t } = useTranslation();
  const accounts = useAccounts();
  const selectAccount = useSelectAccount();
  const removeAccount = useRemoveAccount();
  const refreshAccount = useRefreshAccount();

  const [addOpen, setAddOpen] = useState(false);
  const [removeId, setRemoveId] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  const removeTarget = accounts.data?.find((account) => account.id === removeId);

  function run(action: Promise<unknown>): Promise<boolean> {
    setActionError(null);
    return action.then(
      () => true,
      (error: unknown) => {
        setActionError(error instanceof Error ? error.message : t('error.unknown'));
        return false;
      },
    );
  }

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('accounts.heading')}
        description={t('accounts.description')}
        actions={
          <Button
            variant="primary"
            icon="plus"
            onClick={() => {
              setAddOpen(true);
            }}
          >
            {t('accounts.add')}
          </Button>
        }
      />

      {actionError === null ? null : (
        <p className={pages.errorText} role="alert">
          {actionError}
        </p>
      )}

      {accounts.isError ? (
        <Card title={t('accounts.loadError')} icon="warning">
          <Button
            variant="secondary"
            icon="refresh"
            onClick={() => {
              void accounts.refetch();
            }}
          >
            {t('error.retry')}
          </Button>
        </Card>
      ) : null}

      {accounts.data !== undefined && accounts.data.length === 0 ? (
        <Card
          icon="accounts"
          title={t('accounts.signedOut')}
          description={t('accounts.signedOutBody')}
        />
      ) : null}

      <ul className={pages.list}>
        {accounts.data?.map((account) => (
          <li key={account.id} className={pages.row}>
            <span className={pages.rowMeta} aria-hidden="true">
              <Icon name={KIND_ICONS[account.kind]} size={20} />
            </span>
            <div className={pages.rowMain}>
              <span className={pages.rowTitle}>{account.name}</span>
              <span className={pages.rowMeta}>
                {t(KIND_KEYS[account.kind])} · {account.uuid}
              </span>
            </div>
            {account.selected ? <Badge tone="accent">{t('accounts.selected')}</Badge> : null}
            <div className={pages.rowActions}>
              {account.selected ? null : (
                <Button
                  size="sm"
                  variant="secondary"
                  icon="check"
                  onClick={() => {
                    void run(selectAccount.mutateAsync(account.id));
                  }}
                >
                  {t('accounts.select')}
                </Button>
              )}
              {account.kind === 'offline' ? null : (
                <Button
                  size="sm"
                  variant="ghost"
                  icon="refresh"
                  disabled={refreshAccount.isPending}
                  onClick={() => {
                    void run(refreshAccount.mutateAsync(account.id));
                  }}
                >
                  {t('accounts.refresh')}
                </Button>
              )}
              <Button
                size="sm"
                variant="danger"
                icon="trash"
                onClick={() => {
                  setRemoveId(account.id);
                }}
              >
                {t('accounts.remove')}
              </Button>
            </div>
          </li>
        ))}
      </ul>

      <AddAccountDialog open={addOpen} onClose={() => setAddOpen(false)} />

      <Dialog
        open={removeId !== null}
        title={t('accounts.removeTitle')}
        description={t('accounts.removeBody')}
        closeLabel={t('common.cancel')}
        onClose={() => setRemoveId(null)}
        footer={
          <>
            <Button size="sm" variant="secondary" onClick={() => setRemoveId(null)}>
              {t('common.cancel')}
            </Button>
            <Button
              size="sm"
              variant="danger"
              icon="trash"
              disabled={removeAccount.isPending}
              onClick={() => {
                if (removeTarget === undefined) {
                  return;
                }
                void run(removeAccount.mutateAsync(removeTarget.id)).then((ok) => {
                  if (ok) {
                    setRemoveId(null);
                  }
                });
              }}
            >
              {t('accounts.remove')}
            </Button>
          </>
        }
      />
    </div>
  );
}

interface AddAccountDialogProps {
  readonly open: boolean;
  readonly onClose: () => void;
}

function AddAccountDialog({ open, onClose }: AddAccountDialogProps) {
  const { t } = useTranslation();
  const addOffline = useAddOfflineAccount();
  const addYggdrasil = useAddYggdrasilAccount();
  const beginMicrosoft = useBeginMicrosoftLogin();
  const completeMicrosoft = useCompleteMicrosoftLogin();

  const [method, setMethod] = useState<AddMethod>('microsoft');
  const [username, setUsername] = useState('');
  const [authUrl, setAuthUrl] = useState('https://littleskin.cn/api/yggdrasil');
  const [password, setPassword] = useState('');
  const [challenge, setChallenge] = useState<DeviceCodeChallenge | null>(null);
  const [localError, setLocalError] = useState<string | null>(null);

  const methods: readonly SegmentOption<AddMethod>[] = [
    { value: 'microsoft', label: t('accounts.method.microsoft'), icon: 'accounts' },
    { value: 'offline', label: t('accounts.method.offline'), icon: 'game' },
    { value: 'yggdrasil', label: t('accounts.method.yggdrasil'), icon: 'server' },
  ];

  function reset(): void {
    setUsername('');
    setPassword('');
    setChallenge(null);
    setLocalError(null);
    beginMicrosoft.reset();
    completeMicrosoft.reset();
  }

  function close(): void {
    reset();
    onClose();
  }

  function fail(error: unknown): void {
    setLocalError(error instanceof Error ? error.message : t('error.unknown'));
  }

  const busy = addOffline.isPending || addYggdrasil.isPending || completeMicrosoft.isPending;

  return (
    <Dialog
      open={open}
      title={t('accounts.addTitle')}
      description={t('accounts.addDescription')}
      closeLabel={t('common.cancel')}
      onClose={close}
      footer={
        <>
          <Button size="sm" variant="secondary" onClick={close}>
            {t('common.cancel')}
          </Button>
          {method === 'offline' ? (
            <Button
              size="sm"
              variant="primary"
              icon="plus"
              disabled={busy || username.trim().length === 0}
              onClick={() => {
                setLocalError(null);
                addOffline.mutateAsync(username.trim()).then(close).catch(fail);
              }}
            >
              {t('accounts.addOffline')}
            </Button>
          ) : null}
          {method === 'yggdrasil' ? (
            <Button
              size="sm"
              variant="primary"
              icon="plus"
              disabled={busy || username.trim().length === 0 || password.length === 0}
              onClick={() => {
                setLocalError(null);
                addYggdrasil
                  .mutateAsync({ authUrl: authUrl.trim(), username: username.trim(), password })
                  .then(close)
                  .catch(fail);
              }}
            >
              {t('accounts.addYggdrasil')}
            </Button>
          ) : null}
          {method === 'microsoft' && challenge !== null ? (
            <Button
              size="sm"
              variant="primary"
              icon="check"
              disabled={busy}
              onClick={() => {
                setLocalError(null);
                completeMicrosoft
                  .mutateAsync({ deviceCode: challenge.deviceCode, interval: challenge.interval })
                  .then(close)
                  .catch(fail);
              }}
            >
              {completeMicrosoft.isPending
                ? t('accounts.microsoftWaiting')
                : t('accounts.microsoftApproved')}
            </Button>
          ) : null}
        </>
      }
    >
      <div className={pages.settingBlock}>
        <SegmentedControl
          name="account-method"
          label={t('accounts.addTitle')}
          value={method}
          options={methods}
          onChange={(value) => {
            setMethod(value);
            setLocalError(null);
          }}
        />

        {method === 'offline' ? (
          <div className={pages.field}>
            <label className={pages.fieldLabel} htmlFor="offline-username">
              {t('accounts.username')}
            </label>
            <input
              id="offline-username"
              className={pages.input}
              value={username}
              maxLength={16}
              autoComplete="off"
              onChange={(event) => {
                setUsername(event.target.value);
              }}
            />
            <span className={pages.muted}>{t('accounts.offlineHint')}</span>
          </div>
        ) : null}

        {method === 'yggdrasil' ? (
          <>
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="yggdrasil-url">
                {t('accounts.authUrl')}
              </label>
              <input
                id="yggdrasil-url"
                className={pages.input}
                value={authUrl}
                spellCheck={false}
                onChange={(event) => {
                  setAuthUrl(event.target.value);
                }}
              />
            </div>
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="yggdrasil-username">
                {t('accounts.username')}
              </label>
              <input
                id="yggdrasil-username"
                className={pages.input}
                value={username}
                autoComplete="username"
                onChange={(event) => {
                  setUsername(event.target.value);
                }}
              />
            </div>
            <div className={pages.field}>
              <label className={pages.fieldLabel} htmlFor="yggdrasil-password">
                {t('accounts.password')}
              </label>
              <input
                id="yggdrasil-password"
                className={pages.input}
                type="password"
                value={password}
                autoComplete="current-password"
                onChange={(event) => {
                  setPassword(event.target.value);
                }}
              />
              <span className={pages.muted}>{t('accounts.yggdrasilHint')}</span>
            </div>
          </>
        ) : null}

        {method === 'microsoft' ? (
          <div className={pages.settingBlock}>
            <p className={pages.muted}>{t('accounts.microsoftBody')}</p>
            {challenge === null ? (
              <Button
                variant="secondary"
                icon="external"
                disabled={beginMicrosoft.isPending}
                onClick={() => {
                  setLocalError(null);
                  beginMicrosoft
                    .mutateAsync()
                    .then((value) => {
                      setChallenge(value);
                    })
                    .catch(fail);
                }}
              >
                {beginMicrosoft.isPending
                  ? t('accounts.microsoftWaiting')
                  : t('accounts.microsoftStart')}
              </Button>
            ) : (
              <>
                <p>{t('accounts.microsoftCode')}</p>
                <code className={pages.deviceCode}>{challenge.userCode}</code>
                <a
                  className={pages.muted}
                  href={challenge.verificationUri}
                  target="_blank"
                  rel="noreferrer"
                >
                  {challenge.verificationUri}
                </a>
                <span className={pages.muted}>{challenge.message}</span>
              </>
            )}
          </div>
        ) : null}

        {localError === null ? null : (
          <p className={pages.errorText} role="alert">
            {localError}
          </p>
        )}
      </div>
    </Dialog>
  );
}
