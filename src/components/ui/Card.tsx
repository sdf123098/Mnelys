import type { ReactNode } from 'react';

import { Icon, type IconName } from './Icon';
import styles from './Card.module.css';

interface CardProps {
  readonly title?: ReactNode;
  readonly description?: ReactNode;
  readonly icon?: IconName;
  readonly badge?: ReactNode;
  readonly actions?: ReactNode;
  readonly children?: ReactNode;
  readonly className?: string;
}

/** The one elevated surface in the shell. It is opaque on every platform. */
export function Card({ title, description, icon, badge, actions, children, className }: CardProps) {
  const showHeader = title !== undefined || icon !== undefined;

  return (
    <section className={className === undefined ? styles.card : `${styles.card} ${className}`}>
      {showHeader ? (
        <header className={styles.header}>
          {icon === undefined ? null : (
            <span className={styles.icon} aria-hidden="true">
              <Icon name={icon} size={18} />
            </span>
          )}
          <div className={styles.heading}>
            {title === undefined ? null : <h2>{title}</h2>}
            {description === undefined ? null : <p>{description}</p>}
          </div>
          {badge === undefined ? null : <div className={styles.badge}>{badge}</div>}
        </header>
      ) : null}
      {children === undefined ? null : <div className={styles.body}>{children}</div>}
      {actions === undefined ? null : <footer className={styles.footer}>{actions}</footer>}
    </section>
  );
}
