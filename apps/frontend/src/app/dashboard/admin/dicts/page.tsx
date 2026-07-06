import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminDictsPage } from 'src/pages-layer/admin-dicts';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.dictManagement');
}

export default function Page() {
  return <AdminDictsPage />;
}
