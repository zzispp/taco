'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { useNoticeManagementController } from 'src/features/notice-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { NoticeToolbar } from './toolbar';
import { NoticeDialogs } from './dialogs';
import { NoticeTableSection } from './table-section';

export function AdminNoticesPanel() {
  const { t } = useTranslate('admin');
  const controller = useNoticeManagementController();
  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={t('pages.noticeManagement')} />
      <NoticeToolbar controller={controller} />
      <NoticeTableSection controller={controller} />
      <NoticeDialogs controller={controller} />
    </DashboardContent>
  );
}
