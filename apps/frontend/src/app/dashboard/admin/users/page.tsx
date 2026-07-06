import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminUsersPage } from 'src/pages-layer/admin-users';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.userManagement');
}

export default function Page() {
  return <AdminUsersPage />;
}
