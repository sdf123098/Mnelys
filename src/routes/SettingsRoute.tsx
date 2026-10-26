import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { AccentPicker } from '@/components/ui/AccentPicker';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';
import { SegmentedControl, type SegmentOption } from '@/components/ui/SegmentedControl';
import { Switch } from '@/components/ui/Switch';
import { persistLanguage } from '@/i18n';
import { LANGUAGE_LABELS, matchLanguage, SUPPORTED_LANGUAGES } from '@/i18n/resources';
import type { MirrorKind, MirrorPreference } from '@/ipc/commands';
import {
  useCoreVersion,
  useInstallJava,
  useJavaRuntimes,
  useProbeMirrors,
  useRuntimeInfo,
  useSettings,
  useUpdateSettings,
} from '@/ipc/hooks';
import { describeError } from '@/ipc/messages';
import { usePreferences, type ThemePreference } from '@/state/preferences-store';

import pages from './pages.module.css';

/** Majors the core can provision as verified managed runtimes. */
const JAVA_MAJORS: readonly number[] = [8, 17, 21];

const MIRROR_KEYS = {
  auto: 'settings.mirrorAuto',
  direct: 'settings.mirrorDirect',
  bmclapi: 'settings.mirrorBmclapi',
} as const satisfies Readonly<Record<MirrorPreference, string>>;

const MIRROR_KIND_KEYS = {
  direct: 'settings.mirrorDirect',
  bmclapi: 'settings.mirrorBmclapi',
} as const satisfies Readonly<Record<MirrorKind, string>>;

export function SettingsRoute() {
  const { t, i18n } = useTranslation();
  const runtime = useRuntimeInfo();
  const version = useCoreVersion();
  const settings = useSettings();
  const updateSettings = useUpdateSettings();
  const probeMirrors = useProbeMirrors();
  const java = useJavaRuntimes();
  const installJava = useInstallJava();

  const theme = usePreferences((state) => state.theme);
  const setTheme = usePreferences((state) => state.setTheme);
  const developerMode = usePreferences((state) => state.developerMode);
  const setDeveloperMode = usePreferences((state) => state.setDeveloperMode);

  const currentLanguage = matchLanguage(i18n.resolvedLanguage ?? 'en');

  const [memoryDraft, setMemoryDraft] = useState<string | null>(null);
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const [memorySaved, setMemorySaved] = useState(false);

  const memory = memoryDraft ?? (settings.data === undefined ? '' : String(settings.data.memoryMb));

  const themeOptions: readonly SegmentOption<ThemePreference>[] = [
    { value: 'system', label: t('theme.system'), icon: 'monitor' },
    { value: 'light', label: t('theme.light'), icon: 'sun' },
    { value: 'dark', label: t('theme.dark'), icon: 'moon' },
  ];

  const mirrorOptions: readonly SegmentOption<MirrorPreference>[] = [
    { value: 'auto', label: t(MIRROR_KEYS.auto), icon: 'compass' },
    { value: 'direct', label: t(MIRROR_KEYS.direct), icon: 'globe' },
    { value: 'bmclapi', label: t(MIRROR_KEYS.bmclapi), icon: 'network' },
  ];

  const memoryValue = Number.parseInt(memory, 10);
  const memoryValid =
    !Number.isNaN(memoryValue) &&
    (memoryValue === 0 || (memoryValue >= 512 && memoryValue <= 65536));

  async function changeLanguage(language: string): Promise<void> {
    persistLanguage(language);
    await i18n.changeLanguage(language);
  }

  function saveMemory(): void {
    if (!memoryValid) {
      return;
    }
    setSettingsError(null);
    setMemorySaved(false);
    updateSettings
      .mutateAsync({ memoryMb: memoryValue })
      .then(() => {
        setMemoryDraft(null);
        setMemorySaved(true);
      })
      .catch((cause: unknown) => {
        setSettingsError(describeError(cause, t));
      });
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
          <AccentPicker />
          <p className={pages.caption}>{t('settings.accentDescription')}</p>
        </div>
      </Card>

      <Card title={t('settings.language')} description={t('settings.languageDescription')}>
        <div className={pages.settingBlock}>
          <select
            className={pages.select}
            value={currentLanguage}
            aria-label={t('settings.language')}
            onChange={(event) => {
              void changeLanguage(event.target.value);
            }}
          >
            {SUPPORTED_LANGUAGES.map((language) => (
              <option key={language} value={language} lang={language}>
                {LANGUAGE_LABELS[language]}
              </option>
            ))}
          </select>
        </div>
      </Card>

      <Card title={t('settings.core')} description={t('settings.coreDescription')} icon="database">
        <div className={pages.settingBlock}>
          <SegmentedControl
            name="mirror"
            label={t('settings.mirror')}
            value={settings.data?.mirror ?? 'auto'}
            options={mirrorOptions}
            onChange={(value) => {
              setSettingsError(null);
              updateSettings.mutate(
                { mirror: value },
                {
                  onError: (cause) => {
                    setSettingsError(describeError(cause, t));
                  },
                },
              );
            }}
          />
          <p className={pages.caption}>
            {t('settings.mirrorDescription', {
              active:
                settings.data === undefined ? '—' : t(MIRROR_KIND_KEYS[settings.data.activeMirror]),
            })}
          </p>

          <div className={pages.field}>
            <label className={pages.fieldLabel} htmlFor="memory-mb">
              {t('settings.memory')}
            </label>
            <input
              id="memory-mb"
              className={pages.input}
              type="number"
              min={0}
              max={65536}
              step={256}
              value={memory}
              disabled={settings.isPending}
              onChange={(event) => {
                setMemoryDraft(event.target.value);
                setMemorySaved(false);
              }}
            />
            <span className={pages.muted}>{t('settings.memoryHint')}</span>
            {memoryValid ? null : (
              <span className={pages.errorText}>{t('settings.memoryInvalid')}</span>
            )}
          </div>
          <div className={pages.inlineForm}>
            <Button
              size="sm"
              variant="secondary"
              icon="check"
              disabled={!memoryValid || updateSettings.isPending}
              onClick={saveMemory}
            >
              {t('settings.memorySave')}
            </Button>
            {memorySaved ? (
              <span className={pages.successText}>{t('settings.memorySaved')}</span>
            ) : null}
          </div>

          <div className={pages.field}>
            <span className={pages.fieldLabel}>{t('settings.probe')}</span>
            <div className={pages.inlineForm}>
              <Button
                size="sm"
                variant="secondary"
                icon="network"
                disabled={probeMirrors.isPending}
                onClick={() => {
                  setSettingsError(null);
                  probeMirrors.mutate();
                }}
              >
                {probeMirrors.isPending ? t('settings.probeRunning') : t('settings.probeAction')}
              </Button>
            </div>
            {probeMirrors.data === undefined ? null : (
              <ul className={pages.list}>
                {probeMirrors.data.map((result) => (
                  <li key={result.mirror} className={pages.row}>
                    <div className={pages.rowMain}>
                      <span className={pages.rowTitle}>{t(MIRROR_KIND_KEYS[result.mirror])}</span>
                      <span className={pages.rowMeta}>
                        {result.latencyMs === undefined
                          ? '—'
                          : t('settings.probeLatency', { ms: result.latencyMs })}
                      </span>
                    </div>
                    <Badge tone={result.reachable ? 'success' : 'danger'}>
                      {result.reachable
                        ? t('settings.probeReachable')
                        : t('settings.probeUnreachable')}
                    </Badge>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className={pages.field}>
            <span className={pages.fieldLabel}>{t('settings.root')}</span>
            <code className={pages.codeBox}>{settings.data?.root ?? '—'}</code>
            <span className={pages.muted}>{t('settings.rootDescription')}</span>
          </div>

          {settings.isError ? (
            <p className={pages.errorText} role="alert">
              {t('settings.loadFailed')}
            </p>
          ) : null}
          {settingsError === null ? null : (
            <p className={pages.errorText} role="alert">
              {settingsError}
            </p>
          )}
        </div>
      </Card>

      <Card
        title={t('settings.javaTitle')}
        description={t('settings.javaBody')}
        icon="java"
        badge={
          <Badge tone="neutral">
            {t('settings.javaDetected', { count: java.data?.length ?? 0 })}
          </Badge>
        }
      >
        <div className={pages.settingBlock}>
          <div className={pages.inlineForm}>
            {JAVA_MAJORS.map((major) => (
              <Button
                key={major}
                size="sm"
                variant="secondary"
                icon="downloads"
                disabled={installJava.isPending}
                onClick={() => {
                  installJava.mutate(major);
                }}
              >
                Java {major}
              </Button>
            ))}
          </div>
          <p className={pages.caption}>{t('settings.javaInstallHint')}</p>
          {java.isError ? <p className={pages.errorText}>{t('settings.javaFailed')}</p> : null}
          {java.data !== undefined && java.data.length === 0 ? (
            <p className={pages.muted}>{t('settings.javaEmpty')}</p>
          ) : null}
          <ul className={pages.list}>
            {java.data?.map((runtime) => (
              <li key={runtime.path} className={pages.row}>
                <span className={pages.rowMeta} aria-hidden="true">
                  <Icon name="java" size={18} />
                </span>
                <div className={pages.rowMain}>
                  <span className={pages.rowTitle}>
                    Java {runtime.major} · {runtime.version}
                  </span>
                  <span className={pages.rowMeta}>{runtime.path}</span>
                </div>
                <span className={pages.rowMeta}>{runtime.vendor}</span>
                <Badge tone={runtime.source === 'managed' ? 'accent' : 'neutral'}>
                  {t(`settings.javaSource.${runtime.source}`)}
                </Badge>
              </li>
            ))}
          </ul>
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
