import type { Metadata } from 'next';

import { paths } from 'src/shared/routes/paths';
import { fallbackLng } from 'src/shared/i18n/locales-config';
import { localizePath } from 'src/shared/routes/locale-path';
import { getServerTranslations } from 'src/shared/i18n/server';
import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { Error404Page } from 'src/pages-layer/error-404';

import { ErrorPageProviders } from './providers/error-page-providers';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerTranslations(fallbackLng, 'common');
  return { title: formatErrorDocumentTitle(t('error404.title')) };
}

const DEFAULT_LOCALE_HOME_PATH = localizePath(fallbackLng, paths.home);

export default function NotFound() {
  return (
    <ErrorPageProviders>
      <Error404Page homeHref={DEFAULT_LOCALE_HOME_PATH} />
    </ErrorPageProviders>
  );
}
