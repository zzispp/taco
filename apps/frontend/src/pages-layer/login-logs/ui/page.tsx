'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { useLoginLogController } from 'src/features/audit-log-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';
import { LoginLogToolbar, AdminLoginLogsPanel } from 'src/widgets/admin-login-logs-panel';

export function LoginLogsPage() {
  const { t } = useTranslate('audit');
  const { t: tAdmin } = useTranslate('admin');
  const controller = useLoginLogController();
  const parentLinks = [
    { name: tAdmin('nav.systemMonitor') },
    { name: tAdmin('nav.logManagement') },
  ];
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.loginLogManagement" />
      <DashboardContent>
        <AdminBreadcrumbs
          heading={t('loginLogs')}
          parentLinks={parentLinks}
          onRefresh={controller.actions.refreshPage}
          refreshing={controller.resources.logs.isValidating}
          action={<LoginLogToolbar controller={controller} />}
        />
        <AdminLoginLogsPanel controller={controller} />
      </DashboardContent>
    </>
  );
}
