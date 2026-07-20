import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { AdminJobLogsPage } from 'src/pages-layer/admin-job-logs';

type PageProps = Readonly<{
  params: LocaleRouteParams;
}>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  return getDashboardPageMetadata(await resolveRouteLocale(params), 'pages.jobLogManagement');
}

export default function Page() {
  return <AdminJobLogsPage />;
}
