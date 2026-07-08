import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminOnlinePage } from 'src/pages-layer/admin-online';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.onlineManagement');
}

export default function Page() {
  return <AdminOnlinePage />;
}
