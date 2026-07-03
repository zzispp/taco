import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { RoleManagementView } from 'src/sections/admin/role-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Role management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <RoleManagementView />;
}
