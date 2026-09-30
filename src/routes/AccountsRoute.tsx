import { useTranslation } from 'react-i18next';

import { PlaceholderPanel } from '@/components/PlaceholderPanel';

export function AccountsRoute() {
  const { t } = useTranslation();

  return <PlaceholderPanel heading={t('accounts.heading')} body={t('accounts.empty')} />;
}
