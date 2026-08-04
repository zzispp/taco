'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

import { useJobLogController } from 'src/features/scheduler-management';

import { JobLogToolbar } from './toolbar';
import { JobLogDialogs } from './job-log-dialogs';
import { JobLogTableSection } from './table-section';

export function AdminJobLogsPanel() {
  const { t } = useTranslate('admin');
  const controller = useJobLogController();
  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.jobLogManagement')}
        onRefresh={controller.actions.refreshPage}
        refreshing={controller.resources.logs.isValidating}
        action={<JobLogToolbar controller={controller} />}
      />
      <JobLogTableSection controller={controller} />
      <JobLogDialogs controller={controller} />
    </DashboardContent>
  );
}
