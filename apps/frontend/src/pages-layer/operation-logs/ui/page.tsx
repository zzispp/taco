'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';
import { AdminOperationLogsPanel } from 'src/widgets/admin-operation-logs-panel';

export function OperationLogsPage() {
  const { t } = useTranslate('audit');
  const { t: tAdmin } = useTranslate('admin');
  const parentLinks = [
    { name: tAdmin('nav.systemMonitor') },
    { name: tAdmin('nav.logManagement') },
  ];
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.operationLogManagement" />
      <DashboardContent>
        <AdminBreadcrumbs heading={t('operationLogs')} parentLinks={parentLinks} />
        <AdminOperationLogsPanel />
      </DashboardContent>
    </>
  );
}
