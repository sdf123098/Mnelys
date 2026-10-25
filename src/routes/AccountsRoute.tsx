import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { PageHeader } from '@/components/ui/PageHeader';

import pages from './pages.module.css';

const ACCOUNT_TYPES = [
  {
    id: 'microsoft',
    icon: 'accounts',
    titleKey: 'accounts.types.microsoft.title',
    bodyKey: 'accounts.types.microsoft.body',
  },
  {
    id: 'offline',
    icon: 'game',
    titleKey: 'accounts.types.offline.title',
    bodyKey: 'accounts.types.offline.body',
  },
  {
    id: 'yggdrasil',
    icon: 'server',
    titleKey: 'accounts.types.yggdrasil.title',
    bodyKey: 'accounts.types.yggdrasil.body',
  },
] as const;

export function AccountsRoute() {
  const { t } = useTranslation();

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('accounts.heading')}
        description={t('accounts.description')}
        badge={<Badge tone="warning">{t('common.comingSoon')}</Badge>}
        actions={
          <Button variant="primary" icon="plus" disabled title={t('common.disabledHint')}>
            {t('accounts.add')}
          </Button>
        }
      />

      <Card
        icon="accounts"
        title={t('accounts.signedOut')}
        description={t('accounts.signedOutBody')}
        badge={<Badge tone="neutral">{t('common.m3')}</Badge>}
      />

      <div className={pages.grid}>
        {ACCOUNT_TYPES.map((type) => (
          <Card
            key={type.id}
            icon={type.icon}
            title={t(type.titleKey)}
            description={t(type.bodyKey)}
            badge={
              <Badge tone="neutral">{t('common.availableIn', { milestone: t('common.m3') })}</Badge>
            }
          />
        ))}
      </div>
    </div>
  );
}
