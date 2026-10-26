import { useTranslation } from 'react-i18next';
import { useLocation } from 'react-router-dom';

import { findSection, SECTIONS } from '@/app/navigation';
import { ButtonLink } from '@/components/ui/Button';
import { Icon, type IconName } from '@/components/ui/Icon';
import { IconButton } from '@/components/ui/IconButton';
import { useAccounts } from '@/ipc/hooks';
import { THEME_PREFERENCES, usePreferences, type ThemePreference } from '@/state/preferences-store';
import { useUiStore } from '@/state/ui-store';

import styles from './TopBar.module.css';

const THEME_ICONS: Readonly<Record<ThemePreference, IconName>> = {
  system: 'monitor',
  light: 'sun',
  dark: 'moon',
};

const THEME_LABEL_KEYS = {
  system: 'theme.system',
  light: 'theme.light',
  dark: 'theme.dark',
} as const;

function nextTheme(theme: ThemePreference): ThemePreference {
  const index = THEME_PREFERENCES.indexOf(theme);
  return THEME_PREFERENCES[(index + 1) % THEME_PREFERENCES.length] ?? 'system';
}

export function TopBar() {
  const { t } = useTranslation();
  const { pathname } = useLocation();
  const section = findSection(pathname) ?? SECTIONS[0];

  const theme = usePreferences((state) => state.theme);
  const setTheme = usePreferences((state) => state.setTheme);
  const openTaskCenter = useUiStore((state) => state.openTaskCenter);
  const accounts = useAccounts();
  const signedInAs = accounts.data?.find((account) => account.selected)?.name;

  const themeLabel = t(THEME_LABEL_KEYS[theme]);

  return (
    <header className={styles.topbar}>
      <div className={styles.page}>
        <span className={styles.pageIcon} aria-hidden="true">
          <Icon name={section.icon} size={17} />
        </span>
        <span className={styles.pageTitle}>{t(section.labelKey)}</span>
      </div>

      <span className={styles.spacer} />

      <div className={styles.actions}>
        <IconButton
          icon={THEME_ICONS[theme]}
          label={t('topbar.theme', { theme: themeLabel })}
          onClick={() => {
            setTheme(nextTheme(theme));
          }}
        />
        <IconButton icon="tasks" label={t('topbar.tasks')} onClick={openTaskCenter} />
        <ButtonLink
          to="/accounts"
          variant="ghost"
          size="sm"
          icon="accounts"
          aria-label={signedInAs ?? t('topbar.signedOut')}
          className={styles.account}
        >
          {signedInAs ?? t('topbar.signedOut')}
        </ButtonLink>
      </div>
    </header>
  );
}
