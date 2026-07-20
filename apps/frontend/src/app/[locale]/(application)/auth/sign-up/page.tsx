import type { Metadata } from 'next';

import { getServerTranslations } from 'src/shared/i18n/server';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { SignUpPage } from 'src/pages-layer/sign-up';

// ----------------------------------------------------------------------

type PageProps = Readonly<{
  params: LocaleRouteParams;
}>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const { t } = await getServerTranslations(await resolveRouteLocale(params), 'messages');

  return { title: formatPageDocumentTitle(t('auth.signUp.documentTitle')) };
}

export default function Page() {
  return <SignUpPage />;
}
