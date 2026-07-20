import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { OperationLogsPage } from 'src/pages-layer/operation-logs';

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('pages.operationLogManagement');
}

export default function Page() {
  return <OperationLogsPage />;
}
