import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminJobLogsPage } from 'src/pages-layer/admin-job-logs';

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.jobLogManagement');
}

export default function Page() {
  return <AdminJobLogsPage />;
}
