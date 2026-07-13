'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { useSchedulerController } from 'src/features/scheduler-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { SchedulerToolbar } from './toolbar';
import { SchedulerDialogs } from './scheduler-dialogs';
import { SchedulerTableSection } from './table-section';

export function AdminSchedulerPanel() {
  const { t } = useTranslate('admin');
  const controller = useSchedulerController();
  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={t('pages.jobManagement')} />
      <SchedulerToolbar controller={controller} />
      <SchedulerTableSection controller={controller} />
      <SchedulerDialogs controller={controller} />
    </DashboardContent>
  );
}
