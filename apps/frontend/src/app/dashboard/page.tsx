import type { Metadata } from 'next';

import { redirect } from 'next/navigation';

import { CONFIG } from 'src/shared/config';
import { paths } from 'src/shared/routes/paths';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Dashboard - ${CONFIG.appName}` };

export default function Page() {
  redirect(paths.dashboard.admin.users);
}
