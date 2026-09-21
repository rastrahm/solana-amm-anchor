"use client";

import type { FormEvent } from "react";

export type AmountFieldProps = {
  id: string;
  label: string;
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
};

/**
 * @description Labeled numeric text field for pool amount inputs.
 * @param props.id Input id / accessibility binding.
 * @param props.label Visible label text.
 * @param props.value Controlled string value.
 * @param props.onChange Callback when the value changes.
 * @param props.placeholder Optional placeholder text.
 * @returns Accessible labeled input.
 */
export function AmountField({
  id,
  label,
  value,
  onChange,
  placeholder = "0",
}: AmountFieldProps) {
  return (
    <label className="field" htmlFor={id}>
      <span>{label}</span>
      <input
        id={id}
        name={id}
        inputMode="numeric"
        placeholder={placeholder}
        value={value}
        onChange={(event) => onChange(event.target.value)}
        aria-label={label}
      />
    </label>
  );
}

export type StatusBannerProps = {
  message: string | null;
  tone?: "ok" | "error";
};

/**
 * @description Shows a short status or error message after an action.
 * @param props.message Text to display; hidden when null.
 * @param props.tone Visual tone for success vs error.
 * @returns Status region or null.
 */
export function StatusBanner({ message, tone = "ok" }: StatusBannerProps) {
  if (!message) {
    return null;
  }
  return (
    <p className={`status status-${tone}`} role="status">
      {message}
    </p>
  );
}

/**
 * @description Prevents default form submission for client-handled actions.
 * @param event Form submit event.
 * @param handler Async action to run.
 */
export async function onFormSubmit(
  event: FormEvent<HTMLFormElement>,
  handler: () => Promise<void>
): Promise<void> {
  event.preventDefault();
  await handler();
}
