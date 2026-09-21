"use client";

import { PoolWorkbench } from "@/components/pool/PoolWorkbench";
import { useI18n } from "@/lib/i18n/I18nProvider";

/**
 * @description Home page for the AMM demo: hero copy plus interactive workbench.
 * @returns Landing composition for connect → pool operations.
 */
export default function HomePage() {
  const { t } = useI18n();

  return (
    <main>
      <section className="hero">
        <h1>{t.heroTitle}</h1>
        <p>{t.heroBody}</p>
      </section>
      <PoolWorkbench />
    </main>
  );
}
