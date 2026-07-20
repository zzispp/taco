import type { Metadata } from 'next';

import { getServerTranslations } from 'src/shared/i18n/server';
import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { Error404Page } from 'src/pages-layer/error-404';

// ----------------------------------------------------------------------

export async function generateMetadata({ params }: { params: LocaleRouteParams }): Promise<Metadata> {
  const locale = await resolveRouteLocale(params);
  const { t } = await getServerTranslations(locale, 'common');
  return { title: formatErrorDocumentTitle(t('error404.title')) };
}

export default function Page() {
  return <Error404Page />;
}
