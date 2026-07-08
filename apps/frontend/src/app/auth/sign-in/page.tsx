import type { Metadata } from 'next';

import { getServerTranslations } from 'src/shared/i18n/server';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SignInPage } from 'src/pages-layer/sign-in';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerTranslations('messages');

  return { title: formatPageDocumentTitle(t('auth.signIn.documentTitle')) };
}

export default function Page() {
  return <SignInPage />;
}
