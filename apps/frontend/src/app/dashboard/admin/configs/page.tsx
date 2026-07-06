import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminConfigsPage } from 'src/pages-layer/admin-configs';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.configManagement');
}

export default function Page() {
  return <AdminConfigsPage />;
}
