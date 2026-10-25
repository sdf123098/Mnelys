import type { ReactNode } from 'react';

import { Icon, type IconName } from './Icon';
import styles from './EmptyState.module.css';

interface EmptyStateProps {
  readonly icon: IconName;
  readonly title: ReactNode;
  readonly description: ReactNode;
  readonly action?: ReactNode;
  readonly footnote?: ReactNode;
  readonly className?: string;
}

/**
 * An honest empty state. M1 has no core-backed lists, and pretending otherwise would
 * make every later screenshot a lie, so the shell says what is missing and why.
 */
export function EmptyState({
  icon,
  title,
  description,
  action,
  footnote,
  className,
}: EmptyStateProps) {
  return (
    <section className={className === undefined ? styles.empty : `${styles.empty} ${className}`}>
      <span className={styles.icon} aria-hidden="true">
        <Icon name={icon} size={28} strokeWidth={1.5} />
      </span>
      <h2>{title}</h2>
      <p>{description}</p>
      {action === undefined ? null : <div className={styles.action}>{action}</div>}
      {footnote === undefined ? null : <p className={styles.footnote}>{footnote}</p>}
    </section>
  );
}
