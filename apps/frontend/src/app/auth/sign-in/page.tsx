import type { Metadata } from 'next';

import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SignInPage } from 'src/pages-layer/sign-in';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: formatPageDocumentTitle('Sign in') };

export default function Page() {
  return <SignInPage />;
}
