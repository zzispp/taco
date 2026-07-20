import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminNoticesPage } from 'src/pages-layer/admin-notices';

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.noticeManagement');
}

export default function Page() {
  return <AdminNoticesPage />;
}
