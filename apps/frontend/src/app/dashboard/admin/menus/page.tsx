import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminMenusPage } from 'src/pages-layer/admin-menus';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Menu management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminMenusPage />;
}
