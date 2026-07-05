import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminConfigsPage } from 'src/pages-layer/admin-configs';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `参数设置 | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminConfigsPage />;
}
