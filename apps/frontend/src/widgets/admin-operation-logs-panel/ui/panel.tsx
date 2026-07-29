'use client';

import type { OperationLogController } from 'src/features/audit-log-management';

import { OperationLogToolbar } from './toolbar';
import { OperationLogDialogs } from './dialogs';
import { OperationLogTableSection } from './table-section';

export function AdminOperationLogsPanel({ controller }: { controller: OperationLogController }) {
  return (
    <>
      <OperationLogToolbar controller={controller} />
      <OperationLogTableSection controller={controller} />
      <OperationLogDialogs controller={controller} />
    </>
  );
}
