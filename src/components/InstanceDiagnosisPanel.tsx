import type { TFunction } from 'i18next';
import { Fragment } from 'react';
import { useTranslation } from 'react-i18next';

import { formatBytes } from '@/app/format';
import { Badge, type BadgeTone } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { EmptyState } from '@/components/ui/EmptyState';
import { Icon, type IconName } from '@/components/ui/Icon';
import {
  type DiagnosisFinding,
  type DiagnosisReport,
  type DiagnosisSeverity,
  type DiagnosisSource,
} from '@/ipc/commands';
import { useExportInstanceDiagnostics, useInstanceDiagnosis } from '@/ipc/hooks';

import styles from './InstanceSettingsDialog.module.css';
import pages from '@/routes/pages.module.css';

interface InstanceDiagnosisPanelProps {
  readonly instanceId: string;
}

const SEVERITY_TONES: Record<DiagnosisSeverity, BadgeTone> = {
  critical: 'danger',
  warning: 'warning',
  notice: 'info',
};

const SEVERITY_ICONS: Record<DiagnosisSeverity, IconName> = {
  critical: 'statusError',
  warning: 'warning',
  notice: 'info',
};

/**
 * What the core's rules made of the last launch (plan §19.1).
 *
 * The core sends a rule id, a severity and machine values only, so every sentence here comes
 * from the language catalog — a new rule is a translation, not a string in the runtime. The
 * evidence text is redacted before it crosses the command boundary, which is what lets the
 * same excerpt be written into a bundle someone else can read.
 */
export function InstanceDiagnosisPanel({ instanceId }: InstanceDiagnosisPanelProps) {
  const { t, i18n } = useTranslation();
  const diagnosis = useInstanceDiagnosis(instanceId, true);
  const exportBundle = useExportInstanceDiagnostics();

  const report = diagnosis.data;
  const findings = report?.findings ?? [];
  const lines = new Intl.NumberFormat(i18n.language).format(report?.analyzedLines ?? 0);

  return (
    <>
      <section className={styles.section}>
        <div className={styles.tailHead}>
          <h3 className={styles.sectionTitle}>{t('diagnosis.title')}</h3>
          <Button
            size="sm"
            variant="ghost"
            icon="refresh"
            disabled={diagnosis.isPending}
            onClick={() => {
              void diagnosis.refetch();
            }}
          >
            {t('diagnosis.rerun')}
          </Button>
        </div>
        <p className={pages.muted}>{t('diagnosis.body')}</p>

        {diagnosis.isError ? (
          <p className={pages.errorText}>{t('diagnosis.failed')}</p>
        ) : diagnosis.isPending ? (
          <p className={pages.muted}>{t('common.loading')}</p>
        ) : (
          <>
            <dl className={pages.facts}>
              <dt>{t('diagnosis.sessionTitle')}</dt>
              <dd>{sessionSummary(report, t)}</dd>
              <dt>{t('diagnosis.analyzedTitle')}</dt>
              <dd>{t('diagnosis.analyzed', { lines })}</dd>
              {report?.crashReport === undefined ? null : (
                <>
                  <dt>{t('diagnosis.crashTitle')}</dt>
                  <dd>{report.crashReport}</dd>
                </>
              )}
            </dl>
            {report?.logTruncated === true ? (
              <p className={pages.muted}>{t('diagnosis.truncated')}</p>
            ) : null}
          </>
        )}
      </section>

      {diagnosis.isPending || diagnosis.isError ? null : findings.length === 0 ? (
        <EmptyState
          icon="statusOk"
          title={t('diagnosis.emptyTitle')}
          description={t('diagnosis.emptyBody')}
        />
      ) : (
        <section className={styles.section}>
          <h3 className={styles.sectionTitle}>{t('diagnosis.findingsTitle')}</h3>
          {findings.map((finding) => (
            <Finding key={finding.id} finding={finding} />
          ))}
        </section>
      )}

      <section className={styles.section}>
        <h3 className={styles.sectionTitle}>{t('diagnosis.exportTitle')}</h3>
        <p className={pages.muted}>{t('diagnosis.privacy')}</p>
        <div className={pages.inlineForm}>
          <Button
            size="sm"
            variant="secondary"
            icon="file"
            disabled={exportBundle.isPending || diagnosis.isPending}
            onClick={() => {
              exportBundle.mutate(instanceId);
            }}
          >
            {t('diagnosis.export')}
          </Button>
          {exportBundle.isError ? (
            <span className={pages.errorText}>{t('diagnosis.exportFailed')}</span>
          ) : null}
        </div>
        {exportBundle.data === undefined ? null : (
          <p className={pages.successText} role="status">
            {t('diagnosis.exported', {
              path: exportBundle.data.path,
              size: formatBytes(exportBundle.data.sizeBytes),
            })}
          </p>
        )}
      </section>
    </>
  );
}

/** One rule that fired: what it found, what to do, and the lines that prove it. */
function Finding({ finding }: { readonly finding: DiagnosisFinding }) {
  const { t } = useTranslation();
  const params = PARAM_KEYS.filter((key) => key in finding.params);

  return (
    <article className={styles.subList}>
      <h4 className={styles.subTitle}>
        <Badge tone={SEVERITY_TONES[finding.severity]} icon={SEVERITY_ICONS[finding.severity]}>
          {t(`diagnosis.severities.${finding.severity}`)}
        </Badge>{' '}
        {t(`diagnosis.rules.${finding.id}.title`)}
      </h4>
      <p className={pages.muted}>{t(`diagnosis.rules.${finding.id}.fix`)}</p>
      {params.length === 0 ? null : (
        <dl className={pages.facts}>
          {params.map((key) => (
            <Fragment key={key}>
              <dt>{t(`diagnosis.params.${key}`)}</dt>
              <dd>{finding.params[key]}</dd>
            </Fragment>
          ))}
        </dl>
      )}
      {finding.evidence.map((evidence, index) => (
        <div key={`${evidence.source}-${index}`}>
          <p className={pages.muted}>
            <Icon name={SOURCE_ICONS[evidence.source]} size={13} />{' '}
            {t(`diagnosis.sources.${evidence.source}`)}
            {evidence.path === undefined ? null : ` · ${evidence.path}`}
            {evidence.line === undefined ? null : `:${evidence.line}`}
          </p>
          <pre className={`${pages.codeBox} ${styles.log}`}>{evidence.excerpt}</pre>
        </div>
      ))}
    </article>
  );
}

const SOURCE_ICONS: Record<DiagnosisSource, IconName> = {
  launchLog: 'terminal',
  crashReport: 'bug',
  exitCode: 'cpu',
  instanceConfig: 'settings',
};

/**
 * The values a rule may report, in display order.
 *
 * They are listed as facts next to the sentence rather than interpolated into it: a rule
 * reads a number only when the log happens to carry one, and a translated sentence with a
 * missing slot reads worse than a fact line that simply is not there.
 */
const PARAM_KEYS = ['required', 'supported', 'subtype', 'target', 'mod', 'code'] as const;

function sessionSummary(report: DiagnosisReport | undefined, t: TFunction): string {
  const session = report?.session;
  if (session === undefined) {
    return t('diagnosis.sessionNone');
  }
  const state = t(`diagnosis.states.${session.state}`);
  return session.exitCode === undefined
    ? state
    : `${state} · ${t('diagnosis.exitCode', { code: session.exitCode })}`;
}
