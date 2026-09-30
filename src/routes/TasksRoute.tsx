import { useTranslation } from 'react-i18next';

import { PlaceholderPanel } from '@/components/PlaceholderPanel';

export function TasksRoute() {
  const { t } = useTranslation();

  return <PlaceholderPanel heading={t('tasks.heading')} body={t('tasks.empty')} />;
}
