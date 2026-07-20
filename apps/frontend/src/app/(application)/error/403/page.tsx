import type { Metadata } from 'next';

import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { Error403Page } from 'src/pages-layer/error-403';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: formatErrorDocumentTitle('403 forbidden!') };

export default function Page() {
  return <Error403Page />;
}
