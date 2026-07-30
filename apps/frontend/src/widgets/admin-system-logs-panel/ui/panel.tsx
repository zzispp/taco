'use client';

import type { SystemLogController } from 'src/features/system-log-management';

import { SystemLogDialogs } from './dialogs';
import { SystemLogTableSection } from './table-section';

export function AdminSystemLogsPanel({ controller }: { controller: SystemLogController }) {
  return (
    <>
      <SystemLogTableSection controller={controller} />
      <SystemLogDialogs controller={controller} />
    </>
  );
}
