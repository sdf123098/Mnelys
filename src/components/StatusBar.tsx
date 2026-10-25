import { useTranslation } from 'react-i18next';

import { Icon, type IconName } from '@/components/ui/Icon';
import { useCoreVersion, useRuntimeInfo } from '@/ipc/hooks';

import styles from './StatusBar.module.css';

type CoreState = 'pending' | 'ok' | 'error';

const STATE_ICONS: Readonly<Record<CoreState, IconName>> = {
  pending: 'loading',
  ok: 'statusOk',
  error: 'statusError',
};

function indicatorClass(state: CoreState): string {
  if (state === 'ok') return `${styles.indicator} ${styles.indicatorOk}`;
  if (state === 'error') return `${styles.indicator} ${styles.indicatorError}`;
  return `${styles.indicator} ${styles.indicatorPending}`;
}

/**
 * The boundary's health, always visible.
 *
 * ADR 0006: the core is the source of truth, so the app tells the user when it cannot
 * reach the core instead of showing an empty screen with no explanation.
 */
export function StatusBar() {
  const { t } = useTranslation();
  const version = useCoreVersion();
  const runtime = useRuntimeInfo();

  const state: CoreState = version.isPending ? 'pending' : version.isError ? 'error' : 'ok';
  const label = version.isPending
    ? t('status.checking')
    : version.isError
      ? t('status.coreUnavailable')
      : t('status.coreConnected');

  return (
    <footer
      className={styles.bar}
      role="status"
      aria-live="polite"
      title={version.isError ? t('status.coreUnavailableHint') : undefined}
    >
      <span className={indicatorClass(state)} aria-hidden="true">
        <Icon name={STATE_ICONS[state]} size={14} strokeWidth={2} />
      </span>
      <span>{label}</span>
      {version.data ? (
        <span className={styles.detail}>
          v{version.data.version} ·{' '}
          {t('status.contract', { version: version.data.contractVersion })} · {version.data.profile}
        </span>
      ) : null}
      <span className={styles.spacer} />
      {runtime.data ? (
        <span className={styles.detail}>
          {runtime.data.os}/{runtime.data.arch} · {runtime.data.webview}
        </span>
      ) : null}
    </footer>
  );
}
