import { Icon, type IconName } from './Icon';
import styles from './SegmentedControl.module.css';

export interface SegmentOption<T extends string> {
  readonly value: T;
  readonly label: string;
  readonly icon?: IconName;
}

interface SegmentedControlProps<T extends string> {
  readonly name: string;
  readonly label: string;
  readonly value: T;
  readonly options: readonly SegmentOption<T>[];
  readonly onChange: (value: T) => void;
}

/**
 * Native radio inputs inside one named group. Arrow-key navigation, form semantics and
 * screen-reader grouping come from the platform instead of being re-implemented.
 */
export function SegmentedControl<T extends string>({
  name,
  label,
  value,
  options,
  onChange,
}: SegmentedControlProps<T>) {
  return (
    <fieldset className={styles.group}>
      <legend className={styles.legend}>{label}</legend>
      {options.map((option) => (
        <label key={option.value} className={styles.option}>
          <input
            className={styles.input}
            type="radio"
            name={name}
            value={option.value}
            checked={value === option.value}
            onChange={() => {
              onChange(option.value);
            }}
          />
          <span className={styles.content}>
            {option.icon === undefined ? null : <Icon name={option.icon} size={15} />}
            {option.label}
          </span>
        </label>
      ))}
    </fieldset>
  );
}
