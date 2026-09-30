import { useTranslation } from 'react-i18next';

import { PlaceholderPanel } from '@/components/PlaceholderPanel';

export function InstancesRoute() {
  const { t } = useTranslation();

  return <PlaceholderPanel heading={t('instances.heading')} body={t('instances.empty')} />;
}
