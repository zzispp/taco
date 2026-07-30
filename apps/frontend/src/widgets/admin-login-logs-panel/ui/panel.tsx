'use client';

import type { LoginLogController } from 'src/features/audit-log-management';

import { LoginLogDialogs } from './dialogs';
import { LoginLogTableSection } from './table-section';

export function AdminLoginLogsPanel({ controller }: { controller: LoginLogController }) {
  return (
    <>
      <LoginLogTableSection controller={controller} />
      <LoginLogDialogs controller={controller} />
    </>
  );
}
