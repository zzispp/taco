import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminPostsPage } from 'src/pages-layer/admin-posts';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.postManagement');
}

export default function Page() {
  return <AdminPostsPage />;
}
