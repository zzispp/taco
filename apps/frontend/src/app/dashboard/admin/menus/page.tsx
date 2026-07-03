import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { MenuManagementView } from 'src/sections/admin/menu-management-view';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Menu management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <MenuManagementView />;
}
