import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminMenusPage } from 'src/pages-layer/admin-menus';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.menuManagement');
}

export default function Page() {
  return <AdminMenusPage />;
}
