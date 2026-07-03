import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { AdminApisPage } from 'src/pages-layer/admin-apis';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `API management | Dashboard - ${CONFIG.appName}` };

export default function Page() {
  return <AdminApisPage />;
}
