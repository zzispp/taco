import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { SystemLogsPage } from 'src/pages-layer/system-logs';

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.systemLogManagement');
}

export default function Page() {
  return <SystemLogsPage />;
}
