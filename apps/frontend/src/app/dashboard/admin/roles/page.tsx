import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { AdminRolesPage } from 'src/pages-layer/admin-roles';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.roleManagement');
}

export default function Page() {
  return <AdminRolesPage />;
}
