'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

import { useNoticeManagementController } from 'src/features/notice-management';

import { NoticeDialogs } from './dialogs';
import { NoticeTableSection } from './table-section';

export function AdminNoticesPanel() {
  const { t } = useTranslate('admin');
  const controller = useNoticeManagementController();
  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.noticeManagement')}
        onRefresh={controller.actions.refreshPage}
        refreshing={controller.resources.notices.isValidating}
      />
      <NoticeTableSection controller={controller} />
      <NoticeDialogs controller={controller} />
    </DashboardContent>
  );
}
