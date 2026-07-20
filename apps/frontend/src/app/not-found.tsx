import type { Metadata } from 'next';

import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: formatErrorDocumentTitle('404 page not found!') };

export default function NotFound() {
  return (
    <main>
      <h1>404</h1>
      <p>Page not found.</p>
    </main>
  );
}
