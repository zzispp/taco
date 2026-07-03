import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminRolesPage } from 'src/pages-layer/admin-roles';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Role management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminRolesPage />;
}
