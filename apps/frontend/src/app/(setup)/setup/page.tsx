import type { Metadata } from 'next';

import { getServerTranslations } from 'src/shared/i18n/server';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SetupPage } from 'src/pages-layer/setup';

export async function generateMetadata(): Promise<Metadata> {
  const { t } = await getServerTranslations('setup');
  return { title: formatPageDocumentTitle(t('documentTitle')) };
}

export default function Page() {
  return <SetupPage />;
}
