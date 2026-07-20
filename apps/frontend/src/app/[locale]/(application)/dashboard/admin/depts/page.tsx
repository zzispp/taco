import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { AdminDeptsPage } from 'src/pages-layer/admin-depts';

// ----------------------------------------------------------------------

type PageProps = Readonly<{
  params: LocaleRouteParams;
}>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  return getDashboardPageMetadata(await resolveRouteLocale(params), 'pages.deptManagement');
}

export default function Page() {
  return <AdminDeptsPage />;
}
