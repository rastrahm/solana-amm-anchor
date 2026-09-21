"use client";

import { useI18n } from "@/lib/i18n/I18nProvider";

/**
 * @description Not-found page for unknown app routes.
 * @returns Simple 404 screen with a home link.
 */
export default function NotFoundPage() {
  const { t } = useI18n();

  return (
    <main className="center-page">
      <h1>{t.notFoundTitle}</h1>
      <p>{t.notFoundBody}</p>
      <a href="/">{t.notFoundHome}</a>
    </main>
  );
}
