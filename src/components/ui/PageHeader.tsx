import type { ReactNode } from 'react';

import styles from './PageHeader.module.css';

interface PageHeaderProps {
  readonly title: ReactNode;
  readonly description?: ReactNode;
  readonly badge?: ReactNode;
  readonly actions?: ReactNode;
  readonly eyebrow?: ReactNode;
}

export function PageHeader({ title, description, badge, actions, eyebrow }: PageHeaderProps) {
  return (
    <header className={styles.header}>
      <div className={styles.text}>
        {eyebrow === undefined ? null : <span className={styles.eyebrow}>{eyebrow}</span>}
        <div className={styles.titleRow}>
          <h1>{title}</h1>
          {badge === undefined ? null : badge}
        </div>
        {description === undefined ? null : <p className={styles.description}>{description}</p>}
      </div>
      {actions === undefined ? null : <div className={styles.actions}>{actions}</div>}
    </header>
  );
}
