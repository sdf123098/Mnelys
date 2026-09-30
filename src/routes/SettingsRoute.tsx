import { useTranslation } from 'react-i18next';

import { PlaceholderPanel } from '@/components/PlaceholderPanel';
import { persistLanguage } from '@/i18n';
import { LANGUAGE_LABELS, SUPPORTED_LANGUAGES, type SupportedLanguage } from '@/i18n/resources';
import { useRuntimeInfo } from '@/ipc/hooks';

import styles from './SettingsRoute.module.css';

export function SettingsRoute() {
  const { t, i18n } = useTranslation();
  const runtime = useRuntimeInfo();
  const currentLanguage = i18n.resolvedLanguage;

  async function changeLanguage(language: SupportedLanguage): Promise<void> {
    persistLanguage(language);
    await i18n.changeLanguage(language);
  }

  return (
    <PlaceholderPanel heading={t('settings.heading')} body={t('settings.aboutDescription')}>
      <div className={styles.group}>
        <h2>{t('settings.language')}</h2>
        <p className={styles.hint}>{t('settings.languageDescription')}</p>
        <div className={styles.choices} role="radiogroup" aria-label={t('settings.language')}>
          {SUPPORTED_LANGUAGES.map((language) => {
            const active = currentLanguage === language;
            return (
              <button
                key={language}
                type="button"
                role="radio"
                aria-checked={active}
                className={active ? `${styles.choice} ${styles.choiceActive}` : styles.choice}
                onClick={() => {
                  void changeLanguage(language);
                }}
              >
                {LANGUAGE_LABELS[language]}
              </button>
            );
          })}
        </div>
      </div>

      <div className={styles.group}>
        <h2>{t('settings.about')}</h2>
        <dl className={styles.facts}>
          <dt>{t('home.platform')}</dt>
          <dd>{runtime.data ? `${runtime.data.os}/${runtime.data.arch}` : '—'}</dd>
          <dt>{t('home.webview')}</dt>
          <dd>{runtime.data ? `${runtime.data.webview} (${runtime.data.family})` : '—'}</dd>
          <dt>{t('home.debugBuild')}</dt>
          <dd>{runtime.data ? String(runtime.data.family !== 'unknown') : '—'}</dd>
        </dl>
      </div>
    </PlaceholderPanel>
  );
}
