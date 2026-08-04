'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

import { useSchedulerController } from 'src/features/scheduler-management';

import { SchedulerToolbar } from './toolbar';
import { SchedulerDialogs } from './scheduler-dialogs';
import { SchedulerTableSection } from './table-section';

export function AdminSchedulerPanel() {
  const { t } = useTranslate('admin');
  const controller = useSchedulerController();
  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.jobManagement')}
        onRefresh={controller.actions.refreshPage}
        refreshing={controller.resources.jobs.isValidating}
        action={<SchedulerToolbar controller={controller} />}
      />
      <SchedulerTableSection controller={controller} />
      <SchedulerDialogs controller={controller} />
    </DashboardContent>
  );
}
