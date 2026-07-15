'use client';

import { useLoginLogController } from 'src/features/audit-log-management';

import { LoginLogToolbar } from './toolbar';
import { LoginLogDialogs } from './dialogs';
import { LoginLogTableSection } from './table-section';

export function AdminLoginLogsPanel() {
  const controller = useLoginLogController();
  return (
    <>
      <LoginLogToolbar controller={controller} />
      <LoginLogTableSection controller={controller} />
      <LoginLogDialogs controller={controller} />
    </>
  );
}
