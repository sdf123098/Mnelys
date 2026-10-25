import type { ButtonHTMLAttributes, Ref } from 'react';

import { Icon, type IconName } from './Icon';
import styles from './IconButton.module.css';

interface IconButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  readonly icon: IconName;
  /** Also used as the tooltip, because the control has no visible text. */
  readonly label: string;
  readonly variant?: 'ghost' | 'secondary';
  readonly ref?: Ref<HTMLButtonElement>;
}

export function IconButton({
  icon,
  label,
  variant = 'ghost',
  className,
  type = 'button',
  ref,
  ...rest
}: IconButtonProps) {
  const classes = [styles.button, styles[variant], className].filter(
    (value) => value !== undefined,
  );

  return (
    <button
      ref={ref}
      type={type}
      aria-label={label}
      title={label}
      className={classes.join(' ')}
      {...rest}
    >
      <Icon name={icon} size={18} />
    </button>
  );
}
