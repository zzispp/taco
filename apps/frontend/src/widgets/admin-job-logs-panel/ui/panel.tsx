'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { useJobLogController } from 'src/features/scheduler-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { JobLogToolbar } from './toolbar';
import { JobLogDialogs } from './job-log-dialogs';
import { JobLogTableSection } from './table-section';

export function AdminJobLogsPanel() {
  const { t } = useTranslate('admin');
  const controller = useJobLogController();
  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={t('pages.jobLogManagement')} />
      <JobLogToolbar controller={controller} />
      <JobLogTableSection controller={controller} />
      <JobLogDialogs controller={controller} />
    </DashboardContent>
  );
}
