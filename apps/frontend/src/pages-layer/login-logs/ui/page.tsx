'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';
import { AdminLoginLogsPanel } from 'src/widgets/admin-login-logs-panel';

export function LoginLogsPage() {
  const { t } = useTranslate('audit');
  const { t: tAdmin } = useTranslate('admin');
  const parentLinks = [
    { name: tAdmin('nav.systemMonitor') },
    { name: tAdmin('nav.logManagement') },
  ];
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.loginLogManagement" />
      <DashboardContent>
        <AdminBreadcrumbs heading={t('loginLogs')} parentLinks={parentLinks} />
        <AdminLoginLogsPanel />
      </DashboardContent>
    </>
  );
}
