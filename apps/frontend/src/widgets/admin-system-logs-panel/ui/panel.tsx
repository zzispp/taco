'use client';

import { useSystemLogController } from 'src/features/system-log-management';

import { SystemLogToolbar } from './toolbar';
import { SystemLogDialogs } from './dialogs';
import { SystemLogTableSection } from './table-section';

export function AdminSystemLogsPanel() {
  const controller = useSystemLogController();
  return (
    <>
      <SystemLogToolbar controller={controller} />
      <SystemLogTableSection controller={controller} />
      <SystemLogDialogs controller={controller} />
    </>
  );
}
