import type { Metadata } from 'next';

import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SignUpPage } from 'src/pages-layer/sign-up';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: formatPageDocumentTitle('Sign up') };

export default function Page() {
  return <SignUpPage />;
}
