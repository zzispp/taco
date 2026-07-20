import type { Metadata } from 'next';

import { getServerTranslations } from 'src/shared/i18n/server';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SignUpPage } from 'src/pages-layer/sign-up';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerTranslations('messages');

  return { title: formatPageDocumentTitle(t('auth.signUp.documentTitle')) };
}

export default function Page() {
  return <SignUpPage />;
}
