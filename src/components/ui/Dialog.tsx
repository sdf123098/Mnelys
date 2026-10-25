import { useEffect, useId, useRef, type ReactNode } from 'react';

import { IconButton } from './IconButton';
import styles from './Dialog.module.css';

interface DialogProps {
  readonly open: boolean;
  readonly title: string;
  readonly description?: string | undefined;
  readonly closeLabel: string;
  readonly onClose: () => void;
  readonly children?: ReactNode;
  readonly footer?: ReactNode;
}

/**
 * A modal surface built on the native `<dialog>` element.
 *
 * The platform supplies the top layer, focus containment, Escape handling and focus
 * restoration; this component only keeps React state and the element's `open` state in
 * step. Use it for confirmations, not for navigation (ADR 0006).
 */
export function Dialog({
  open,
  title,
  description,
  closeLabel,
  onClose,
  children,
  footer,
}: DialogProps) {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const descriptionId = useId();

  useEffect(() => {
    const dialog = dialogRef.current;
    if (dialog === null) {
      return;
    }

    if (open && !dialog.open) {
      dialog.showModal();
    } else if (!open && dialog.open) {
      dialog.close();
    }
  }, [open]);

  return (
    <dialog
      ref={dialogRef}
      className={styles.dialog}
      aria-labelledby={titleId}
      aria-describedby={description === undefined ? undefined : descriptionId}
      onCancel={(event) => {
        event.preventDefault();
        onClose();
      }}
      onClick={(event) => {
        if (event.target === dialogRef.current) {
          onClose();
        }
      }}
    >
      <div className={styles.panel}>
        <header className={styles.header}>
          <div className={styles.heading}>
            <h2 id={titleId}>{title}</h2>
            {description === undefined ? null : <p id={descriptionId}>{description}</p>}
          </div>
          <IconButton icon="close" label={closeLabel} onClick={onClose} />
        </header>
        {children === undefined ? null : <div className={styles.body}>{children}</div>}
        {footer === undefined ? null : <footer className={styles.footer}>{footer}</footer>}
      </div>
    </dialog>
  );
}
