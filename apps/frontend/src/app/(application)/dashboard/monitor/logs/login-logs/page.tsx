import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { LoginLogsPage } from 'src/pages-layer/login-logs';

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.loginLogManagement');
}

export default function Page() {
  return <LoginLogsPage />;
}
