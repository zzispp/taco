import type { Metadata } from 'next';

import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { Error500Page } from 'src/pages-layer/error-500';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: formatErrorDocumentTitle('500 Internal server error!'),
};

export default function Page() {
  return <Error500Page />;
}
