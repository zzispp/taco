import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { FileManagerPage } from 'src/pages-layer/file-manager';

type PageProps = Readonly<{ params: LocaleRouteParams }>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  return getDashboardPageMetadata(await resolveRouteLocale(params), 'file.managerTitle');
}

export default function Page() {
  return <FileManagerPage />;
}
