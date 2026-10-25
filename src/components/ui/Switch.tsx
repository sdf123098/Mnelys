import { useId } from 'react';

import styles from './Switch.module.css';

interface SwitchProps {
  readonly label: string;
  readonly description?: string;
  readonly checked: boolean;
  readonly onChange: (checked: boolean) => void;
  readonly disabled?: boolean;
}

export function Switch({ label, description, checked, onChange, disabled = false }: SwitchProps) {
  const inputId = useId();
  const descriptionId = useId();

  return (
    <div className={styles.row}>
      <span className={styles.text}>
        <label className={styles.label} htmlFor={inputId}>
          {label}
        </label>
        {description === undefined ? null : (
          <span className={styles.description} id={descriptionId}>
            {description}
          </span>
        )}
      </span>
      <label className={styles.control} htmlFor={inputId}>
        <input
          className={styles.input}
          id={inputId}
          type="checkbox"
          role="switch"
          checked={checked}
          disabled={disabled}
          aria-describedby={description === undefined ? undefined : descriptionId}
          onChange={(event) => {
            onChange(event.currentTarget.checked);
          }}
        />
        <span className={styles.track} aria-hidden="true">
          <span className={styles.thumb} />
        </span>
      </label>
    </div>
  );
}
