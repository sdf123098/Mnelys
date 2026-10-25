import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { PageHeader } from '@/components/ui/PageHeader';

import pages from './pages.module.css';

export function InstancesRoute() {
  const { t } = useTranslation();

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('instances.heading')}
        description={t('instances.description')}
        badge={<Badge tone="warning">{t('common.comingSoon')}</Badge>}
        actions={
          <Button variant="primary" icon="plus" disabled title={t('common.disabledHint')}>
            {t('instances.create')}
          </Button>
        }
      />

      <EmptyState
        icon="instances"
        title={t('instances.emptyTitle')}
        description={t('instances.emptyBody')}
        footnote={t('common.availableIn', { milestone: t('common.m2') })}
      />

      <Card
        title={t('instances.plannedTitle')}
        description={t('instances.plannedBody')}
        icon="layers"
      >
        <div className={pages.notice}>
          <p>{t('instances.plannedNotice')}</p>
        </div>
      </Card>
    </div>
  );
}
