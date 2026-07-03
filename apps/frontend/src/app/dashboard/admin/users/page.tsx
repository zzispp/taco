import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminUsersPage } from 'src/pages-layer/admin-users';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `User management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminUsersPage />;
}
