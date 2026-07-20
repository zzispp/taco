import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { AdminRolesPage } from 'src/pages-layer/admin-roles';

// ----------------------------------------------------------------------

type PageProps = Readonly<{
  params: LocaleRouteParams;
}>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  return getDashboardPageMetadata(await resolveRouteLocale(params), 'pages.roleManagement');
}

export default function Page() {
  return <AdminRolesPage />;
}
