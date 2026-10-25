import type { AnchorHTMLAttributes, ButtonHTMLAttributes, ReactNode } from 'react';
import { Link } from 'react-router-dom';

import { Icon, type IconName } from './Icon';
import styles from './Button.module.css';

export type ButtonVariant = 'primary' | 'secondary' | 'ghost' | 'danger';
export type ButtonSize = 'sm' | 'md';

interface ButtonBaseProps {
  readonly variant?: ButtonVariant | undefined;
  readonly size?: ButtonSize | undefined;
  readonly icon?: IconName | undefined;
  readonly iconRight?: IconName | undefined;
  readonly children: ReactNode;
  readonly className?: string | undefined;
}

type ButtonProps = ButtonBaseProps & Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'>;

type ButtonLinkProps = ButtonBaseProps &
  Omit<AnchorHTMLAttributes<HTMLAnchorElement>, 'children' | 'href'> & {
    readonly to: string;
  };

function join(...values: ReadonlyArray<string | undefined>): string {
  return values.filter((value) => value !== undefined).join(' ');
}

function content({ icon, iconRight, children }: ButtonBaseProps): ReactNode {
  return (
    <>
      {icon === undefined ? null : <Icon name={icon} size={16} />}
      <span>{children}</span>
      {iconRight === undefined ? null : <Icon name={iconRight} size={16} />}
    </>
  );
}

export function Button({
  variant = 'secondary',
  size = 'md',
  icon,
  iconRight,
  children,
  className,
  type = 'button',
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={join(styles.button, styles[variant], styles[size], className)}
      {...rest}
    >
      {content({ icon, iconRight, children })}
    </button>
  );
}

/** A router link that keeps button styling and never becomes a second navigation model. */
export function ButtonLink({
  variant = 'secondary',
  size = 'md',
  icon,
  iconRight,
  children,
  className,
  ...rest
}: ButtonLinkProps) {
  return (
    <Link className={join(styles.button, styles[variant], styles[size], className)} {...rest}>
      {content({ icon, iconRight, children })}
    </Link>
  );
}
