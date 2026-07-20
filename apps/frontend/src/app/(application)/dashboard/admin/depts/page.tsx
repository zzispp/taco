import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminDeptsPage } from 'src/pages-layer/admin-depts';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.deptManagement');
}

export default function Page() {
  return <AdminDeptsPage />;
}
