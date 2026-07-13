import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminJobsPage } from 'src/pages-layer/admin-jobs';

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.jobManagement');
}

export default function Page() {
  return <AdminJobsPage />;
}
