"use client";

import { useI18n } from "@/lib/i18n/I18nProvider";

/**
 * @description Route-level error UI for the main app segment.
 * @param props.error Thrown error instance.
 * @param props.reset Callback to retry rendering the segment.
 * @returns Recoverable error screen.
 */
export default function ErrorPage({
  error,
  reset,
}: {
  error: Error & { digest?: string };
  reset: () => void;
}) {
  const { t } = useI18n();

  return (
    <main className="center-page">
      <h1>{t.errorTitle}</h1>
      <p>{error.message}</p>
      <button type="button" onClick={reset}>
        {t.errorRetry}
      </button>
    </main>
  );
}
