'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

import { useOperationLogController } from 'src/features/audit-log-management';

import {
  OperationLogToolbar,
  AdminOperationLogsPanel,
} from 'src/widgets/admin-operation-logs-panel';

export function OperationLogsPage() {
  const { t } = useTranslate('audit');
  const { t: tAdmin } = useTranslate('admin');
  const controller = useOperationLogController();
  const parentLinks = [
    { name: tAdmin('nav.systemMonitor') },
    { name: tAdmin('nav.logManagement') },
  ];
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.operationLogManagement" />
      <DashboardContent>
        <AdminBreadcrumbs
          heading={t('operationLogs')}
          parentLinks={parentLinks}
          onRefresh={controller.actions.refreshPage}
          refreshing={controller.resources.logs.isValidating}
          action={<OperationLogToolbar controller={controller} />}
        />
        <AdminOperationLogsPanel controller={controller} />
      </DashboardContent>
    </>
  );
}
