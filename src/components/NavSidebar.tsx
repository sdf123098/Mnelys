import { useTranslation } from 'react-i18next';
import { NavLink } from 'react-router-dom';

import styles from './NavSidebar.module.css';

const SECTIONS = [
  { to: '/instances', labelKey: 'nav.instances' },
  { to: '/accounts', labelKey: 'nav.accounts' },
  { to: '/tasks', labelKey: 'nav.tasks' },
  { to: '/settings', labelKey: 'nav.settings' },
] as const;

export function NavSidebar() {
  const { t } = useTranslation();

  return (
    <nav className={styles.sidebar} aria-label={t('app.name')}>
      <div className={styles.brand}>
        <span className={styles.brandName}>{t('app.chineseName')}</span>
        <span className={styles.brandMeta}>{t('app.name')}</span>
      </div>
      <ul className={styles.list}>
        {SECTIONS.map((section) => (
          <li key={section.to}>
            <NavLink
              to={section.to}
              className={({ isActive }) =>
                isActive ? `${styles.link} ${styles.linkActive}` : styles.link
              }
            >
              {t(section.labelKey)}
            </NavLink>
          </li>
        ))}
      </ul>
    </nav>
  );
}
