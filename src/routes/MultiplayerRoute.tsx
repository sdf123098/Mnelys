import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { EmptyState } from '@/components/ui/EmptyState';
import { PageHeader } from '@/components/ui/PageHeader';

import pages from './pages.module.css';

export function MultiplayerRoute() {
  const { t } = useTranslation();

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('multiplayer.heading')}
        description={t('multiplayer.description')}
        badge={<Badge tone="warning">{t('common.comingSoon')}</Badge>}
      />

      <EmptyState
        icon="multiplayer"
        title={t('multiplayer.emptyTitle')}
        description={t('multiplayer.emptyBody')}
        action={
          <Button variant="primary" icon="plus" disabled title={t('common.disabledHint')}>
            {t('multiplayer.createRoom')}
          </Button>
        }
        footnote={t('common.availableIn', { milestone: t('common.m7') })}
      />

      <Card
        icon="shield"
        title={t('multiplayer.safetyHeading')}
        description={t('multiplayer.safetyBody')}
        badge={<Badge tone="info">{t('common.m7')}</Badge>}
      />
    </div>
  );
}
