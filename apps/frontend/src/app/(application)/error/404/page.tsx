import type { Metadata } from 'next';

import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { Error404Page } from 'src/pages-layer/error-404';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: formatErrorDocumentTitle('404 page not found!') };

export default function Page() {
  return <Error404Page />;
}
