import { useTranslation } from 'react-i18next';
import { NavLink } from 'react-router-dom';

import { SECTIONS } from '@/app/navigation';
import { Badge } from '@/components/ui/Badge';
import { Icon } from '@/components/ui/Icon';
import { usePreferences } from '@/state/preferences-store';

import styles from './NavSidebar.module.css';

export function NavSidebar() {
  const { t } = useTranslation();
  const developerMode = usePreferences((state) => state.developerMode);

  const sections = SECTIONS.filter(
    (section) => section.inNav && (!section.developerOnly || developerMode),
  );

  return (
    <nav className={styles.sidebar} aria-label={t('nav.label')}>
      <div className={styles.brand}>
        <img className={styles.brandMark} src="/app-icon.png" alt="" width={36} height={36} />
        <span className={styles.brandText}>
          <span className={styles.brandName}>{t('app.name')}</span>
        </span>
      </div>

      <ul className={styles.list}>
        {sections.map((section) => (
          <li key={section.id}>
            <NavLink
              to={section.to}
              title={t(section.labelKey)}
              aria-label={t(section.labelKey)}
              className={({ isActive }) =>
                isActive ? `${styles.link} ${styles.linkActive}` : styles.link
              }
            >
              <span className={styles.icon} aria-hidden="true">
                <Icon name={section.icon} size={19} />
              </span>
              <span className={styles.label}>{t(section.labelKey)}</span>
            </NavLink>
          </li>
        ))}
      </ul>

      <div className={styles.footer}>
        <Badge tone="neutral">{t('app.stage')}</Badge>
      </div>
    </nav>
  );
}
