import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminDeptsPage } from 'src/pages-layer/admin-depts';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `部门管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminDeptsPage />;
}
