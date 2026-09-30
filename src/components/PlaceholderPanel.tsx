import { useId } from 'react';
import type { ReactNode } from 'react';

import styles from './PlaceholderPanel.module.css';

interface PlaceholderPanelProps {
  /** Already translated by the caller, so this component stays presentational. */
  readonly heading: string;
  readonly body: string;
  readonly children?: ReactNode;
}

/** Shared surface for a section that exists in the shell but has no slice yet. */
export function PlaceholderPanel({ heading, body, children }: PlaceholderPanelProps) {
  const headingId = useId();

  return (
    <section className={styles.panel} aria-labelledby={headingId}>
      <h1 id={headingId}>{heading}</h1>
      <p className={styles.body}>{body}</p>
      {children}
    </section>
  );
}
