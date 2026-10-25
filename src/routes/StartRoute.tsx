import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button, ButtonLink } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { Icon } from '@/components/ui/Icon';
import { PageHeader } from '@/components/ui/PageHeader';

import pages from './pages.module.css';

const NEXT_STEPS = [
  {
    id: 'metadata',
    titleKey: 'start.steps.metadata.title',
    bodyKey: 'start.steps.metadata.body',
  },
  {
    id: 'instance',
    titleKey: 'start.steps.instance.title',
    bodyKey: 'start.steps.instance.body',
  },
  {
    id: 'launch',
    titleKey: 'start.steps.launch.title',
    bodyKey: 'start.steps.launch.body',
  },
] as const;

export function StartRoute() {
  const { t } = useTranslation();

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('start.heading')}
        description={t('start.description')}
        badge={<Badge tone="accent">{t('common.m1')}</Badge>}
      />

      <section className={pages.hero}>
        <div className={pages.heroBody}>
          <h2>{t('start.welcomeTitle')}</h2>
          <p>{t('start.welcomeBody')}</p>
          <div className={pages.heroActions}>
            <Button variant="primary" icon="play" disabled title={t('start.launchDisabled')}>
              {t('start.launch')}
            </Button>
            <ButtonLink to="/instances" variant="secondary" icon="instances">
              {t('start.openInstances')}
            </ButtonLink>
          </div>
          <p className={pages.caption}>{t('start.launchDisabled')}</p>
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
          badge={<Badge tone="neutral">{t('start.cards.account.value')}</Badge>}
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
          badge={<Badge tone="neutral">{t('start.cards.instances.value')}</Badge>}
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
          badge={<Badge tone="neutral">{t('start.cards.downloads.value')}</Badge>}
          actions={
            <ButtonLink to="/downloads" variant="ghost" size="sm" iconRight="arrowRight">
              {t('common.open')}
            </ButtonLink>
          }
        />
      </div>

      <Card title={t('start.nextHeading')} description={t('start.nextBody')}>
        <ol className={pages.steps}>
          {NEXT_STEPS.map((step, index) => (
            <li key={step.id} className={pages.step}>
              <span className={pages.stepIndex} aria-hidden="true">
                {index + 1}
              </span>
              <div>
                <h3>{t(step.titleKey)}</h3>
                <p>{t(step.bodyKey)}</p>
              </div>
            </li>
          ))}
        </ol>
      </Card>
    </div>
  );
}
