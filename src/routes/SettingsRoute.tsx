import { useTranslation } from 'react-i18next';

import { Card } from '@/components/ui/Card';
import { PageHeader } from '@/components/ui/PageHeader';
import { SegmentedControl, type SegmentOption } from '@/components/ui/SegmentedControl';
import { Switch } from '@/components/ui/Switch';
import { isSupportedLanguage, persistLanguage } from '@/i18n';
import { LANGUAGE_LABELS, SUPPORTED_LANGUAGES, type SupportedLanguage } from '@/i18n/resources';
import { useCoreVersion, useRuntimeInfo } from '@/ipc/hooks';
import { usePreferences, type ThemePreference } from '@/state/preferences-store';

import pages from './pages.module.css';

export function SettingsRoute() {
  const { t, i18n } = useTranslation();
  const runtime = useRuntimeInfo();
  const version = useCoreVersion();

  const theme = usePreferences((state) => state.theme);
  const setTheme = usePreferences((state) => state.setTheme);
  const developerMode = usePreferences((state) => state.developerMode);
  const setDeveloperMode = usePreferences((state) => state.setDeveloperMode);

  const currentLanguage: SupportedLanguage = isSupportedLanguage(i18n.resolvedLanguage)
    ? i18n.resolvedLanguage
    : 'en';

  const languageOptions: readonly SegmentOption<SupportedLanguage>[] = SUPPORTED_LANGUAGES.map(
    (language) => ({ value: language, label: LANGUAGE_LABELS[language] }),
  );

  const themeOptions: readonly SegmentOption<ThemePreference>[] = [
    { value: 'system', label: t('theme.system'), icon: 'monitor' },
    { value: 'light', label: t('theme.light'), icon: 'sun' },
    { value: 'dark', label: t('theme.dark'), icon: 'moon' },
  ];

  async function changeLanguage(language: SupportedLanguage): Promise<void> {
    persistLanguage(language);
    await i18n.changeLanguage(language);
  }

  return (
    <div className={pages.narrow}>
      <PageHeader title={t('settings.heading')} description={t('settings.description')} />

      <Card title={t('settings.appearance')} description={t('settings.appearanceDescription')}>
        <div className={pages.settingBlock}>
          <SegmentedControl
            name="theme"
            label={t('theme.label')}
            value={theme}
            options={themeOptions}
            onChange={setTheme}
          />
          <p className={pages.caption}>{t('theme.description')}</p>
        </div>
      </Card>

      <Card title={t('settings.language')} description={t('settings.languageDescription')}>
        <div className={pages.settingBlock}>
          <SegmentedControl
            name="language"
            label={t('settings.language')}
            value={currentLanguage}
            options={languageOptions}
            onChange={(language) => {
              void changeLanguage(language);
            }}
          />
        </div>
      </Card>

      <Card title={t('nav.developer')}>
        <Switch
          label={t('settings.developer')}
          description={t('settings.developerDescription')}
          checked={developerMode}
          onChange={setDeveloperMode}
        />
      </Card>

      <Card title={t('settings.about')} description={t('settings.aboutDescription')}>
        <dl className={pages.facts}>
          <dt>{t('settings.platform')}</dt>
          <dd>{runtime.data ? `${runtime.data.os}/${runtime.data.arch}` : '—'}</dd>
          <dt>{t('settings.webview')}</dt>
          <dd>{runtime.data ? `${runtime.data.webview} (${runtime.data.family})` : '—'}</dd>
          <dt>{t('settings.coreVersion')}</dt>
          <dd>{version.data ? `v${version.data.version}` : '—'}</dd>
          <dt>{t('settings.contract')}</dt>
          <dd>{version.data ? `v${version.data.contractVersion}` : '—'}</dd>
          <dt>{t('settings.icons')}</dt>
          <dd>Lucide (ISC)</dd>
        </dl>
        <p className={pages.caption}>{t('settings.iconsDescription')}</p>
      </Card>
    </div>
  );
}
