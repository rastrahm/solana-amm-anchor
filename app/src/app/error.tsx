"use client";

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
  return (
    <main className="center-page">
      <h1>Something went wrong</h1>
      <p>{error.message}</p>
      <button type="button" onClick={reset}>
        Try again
      </button>
    </main>
  );
}
