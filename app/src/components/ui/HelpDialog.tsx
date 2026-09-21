"use client";

import { useEffect, useId, useRef, type ReactNode } from "react";
import { useI18n } from "@/lib/i18n/I18nProvider";

export type HelpDialogProps = {
  open: boolean;
  onClose: () => void;
};

/**
 * @description Modal help content for the AMM demo in the active locale.
 * @param props.open Whether the dialog is visible.
 * @param props.onClose Close handler (backdrop, Escape, or close button).
 * @returns Accessible dialog or null when closed.
 */
export function HelpDialog({ open, onClose }: HelpDialogProps) {
  const { t } = useI18n();
  const titleId = useId();
  const closeRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (!open) {
      return;
    }
    closeRef.current?.focus();
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        onClose();
      }
    }
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) {
    return null;
  }

  const { help } = t;

  return (
    <div className="help-overlay" role="presentation" onClick={onClose}>
      <div
        className="help-dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        onClick={(event) => event.stopPropagation()}
      >
        <div className="help-dialog-header">
          <h2 id={titleId}>{t.helpTitle}</h2>
          <button
            ref={closeRef}
            type="button"
            className="icon-btn"
            onClick={onClose}
            aria-label={t.helpClose}
          >
            ×
          </button>
        </div>
        <div className="help-dialog-body">
          <p>{help.intro}</p>
          <HelpSection title={help.stepsTitle}>
            <ol>
              {help.steps.map((step) => (
                <li key={step}>{step}</li>
              ))}
            </ol>
          </HelpSection>
          <HelpSection title={help.fieldsTitle}>
            <ul>
              {help.fields.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </HelpSection>
          <HelpSection title={help.phantomTitle}>
            <ol>
              {help.phantom.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ol>
          </HelpSection>
          <HelpSection title={help.tipsTitle}>
            <ul>
              {help.tips.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
          </HelpSection>
        </div>
      </div>
    </div>
  );
}

function HelpSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="help-section">
      <h3>{title}</h3>
      {children}
    </section>
  );
}
