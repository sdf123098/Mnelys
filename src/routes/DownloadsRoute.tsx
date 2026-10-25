import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { PageHeader } from '@/components/ui/PageHeader';

import pages from './pages.module.css';

export function DownloadsRoute() {
  const { t } = useTranslation();

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('downloads.heading')}
        description={t('downloads.description')}
        badge={<Badge tone="warning">{t('common.comingSoon')}</Badge>}
      />

      <EmptyState
        icon="downloads"
        title={t('downloads.emptyTitle')}
        description={t('downloads.emptyBody')}
        footnote={t('common.availableIn', { milestone: t('common.m2') })}
      />

      <header className={pages.sectionHeader}>
        <h2>{t('downloads.providersHeading')}</h2>
        <p>{t('downloads.providersBody')}</p>
      </header>

      <div className={pages.grid}>
        <Card
          icon="database"
          title={t('downloads.providers.mojang.title')}
          description={t('downloads.providers.mojang.body')}
          badge={
            <Badge tone="neutral">{t('common.availableIn', { milestone: t('common.m2') })}</Badge>
          }
        />
        <Card
          icon="java"
          title={t('downloads.providers.java.title')}
          description={t('downloads.providers.java.body')}
          badge={
            <Badge tone="neutral">{t('common.availableIn', { milestone: t('common.m2') })}</Badge>
          }
        />
        <Card
          icon="globe"
          title={t('downloads.providers.content.title')}
          description={t('downloads.providers.content.body')}
          badge={
            <Badge tone="neutral">{t('common.availableIn', { milestone: t('common.m6') })}</Badge>
          }
        />
      </div>
    </div>
  );
}
