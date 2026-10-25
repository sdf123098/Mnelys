import { useTranslation } from 'react-i18next';

import { Badge } from '@/components/ui/Badge';
import { Button } from '@/components/ui/Button';
import { Card } from '@/components/ui/Card';
import { PageHeader } from '@/components/ui/PageHeader';
import { usePreferences } from '@/state/preferences-store';

import pages from './pages.module.css';

const TOOLS = [
  {
    id: 'build',
    icon: 'terminal',
    titleKey: 'developer.tools.build.title',
    bodyKey: 'developer.tools.build.body',
  },
  {
    id: 'debug',
    icon: 'bug',
    titleKey: 'developer.tools.debug.title',
    bodyKey: 'developer.tools.debug.body',
  },
  {
    id: 'daemon',
    icon: 'server',
    titleKey: 'developer.tools.daemon.title',
    bodyKey: 'developer.tools.daemon.body',
  },
] as const;

export function DeveloperRoute() {
  const { t } = useTranslation();
  const developerMode = usePreferences((state) => state.developerMode);
  const setDeveloperMode = usePreferences((state) => state.setDeveloperMode);

  return (
    <div className={pages.page}>
      <PageHeader
        title={t('developer.heading')}
        description={t('developer.description')}
        badge={
          <Badge tone={developerMode ? 'success' : 'neutral'}>
            {developerMode ? t('developer.enabledTitle') : t('developer.gateTitle')}
          </Badge>
        }
        actions={
          developerMode ? (
            <Button
              variant="secondary"
              icon="close"
              onClick={() => {
                setDeveloperMode(false);
              }}
            >
              {t('developer.disableAction')}
            </Button>
          ) : undefined
        }
      />

      {developerMode ? (
        <>
          <header className={pages.sectionHeader}>
            <h2>{t('developer.toolsTitle')}</h2>
            <p>{t('developer.enabledBody')}</p>
          </header>
          <div className={pages.grid}>
            {TOOLS.map((tool) => (
              <Card
                key={tool.id}
                icon={tool.icon}
                title={t(tool.titleKey)}
                description={t(tool.bodyKey)}
              />
            ))}
          </div>
        </>
      ) : (
        <Card
          icon="developer"
          title={t('developer.gateTitle')}
          description={t('developer.gateBody')}
          actions={
            <Button
              variant="primary"
              icon="check"
              onClick={() => {
                setDeveloperMode(true);
              }}
            >
              {t('developer.gateAction')}
            </Button>
          }
        />
      )}
    </div>
  );
}
