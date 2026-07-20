import type { Metadata } from 'next';

import { getServerTranslations } from 'src/shared/i18n/server';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { SetupPage } from 'src/pages-layer/setup';

type PageProps = Readonly<{
  params: LocaleRouteParams;
}>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const { t } = await getServerTranslations(await resolveRouteLocale(params), 'setup');
  return { title: formatPageDocumentTitle(t('documentTitle')) };
}

export default function Page() {
  return <SetupPage />;
}
