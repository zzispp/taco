'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';
import { AdminSystemLogsPanel } from 'src/widgets/admin-system-logs-panel';

export function SystemLogsPage() {
  const { t } = useTranslate('systemLog');
  const { t: tAdmin } = useTranslate('admin');
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.systemLogManagement" />
      <DashboardContent>
        <AdminBreadcrumbs
          heading={t('title')}
          parentLinks={[
            { name: tAdmin('nav.systemMonitor') },
            { name: tAdmin('nav.logManagement') },
          ]}
        />
        <AdminSystemLogsPanel />
      </DashboardContent>
    </>
  );
}
