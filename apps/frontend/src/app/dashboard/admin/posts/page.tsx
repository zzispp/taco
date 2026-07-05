import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminPostsPage } from 'src/pages-layer/admin-posts';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `岗位管理 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminPostsPage />;
}
