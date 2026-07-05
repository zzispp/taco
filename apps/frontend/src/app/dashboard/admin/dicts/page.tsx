import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminDictsPage } from 'src/pages-layer/admin-dicts';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `字典管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminDictsPage />;
}
