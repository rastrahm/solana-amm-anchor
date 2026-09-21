/**
 * @description Not-found page for unknown app routes.
 * @returns Simple 404 screen with a home link.
 */
export default function NotFoundPage() {
  return (
    <main className="center-page">
      <h1>Page not found</h1>
      <p>That route does not exist in the AMM demo.</p>
      <a href="/">Back home</a>
    </main>
  );
}
