'use client';

import { useOperationLogController } from 'src/features/audit-log-management';

import { OperationLogToolbar } from './toolbar';
import { OperationLogDialogs } from './dialogs';
import { OperationLogTableSection } from './table-section';

export function AdminOperationLogsPanel() {
  const controller = useOperationLogController();
  return (
    <>
      <OperationLogToolbar controller={controller} />
      <OperationLogTableSection controller={controller} />
      <OperationLogDialogs controller={controller} />
    </>
  );
}
