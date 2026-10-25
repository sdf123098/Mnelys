import type { ReactNode } from 'react';

import { Icon, type IconName } from './Icon';
import styles from './Badge.module.css';

export type BadgeTone = 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'info';

interface BadgeProps {
  readonly tone?: BadgeTone;
  readonly icon?: IconName;
  readonly children: ReactNode;
}

export function Badge({ tone = 'neutral', icon, children }: BadgeProps) {
  return (
    <span className={`${styles.badge} ${styles[tone]}`}>
      {icon === undefined ? null : <Icon name={icon} size={12} strokeWidth={2} />}
      {children}
    </span>
  );
}
