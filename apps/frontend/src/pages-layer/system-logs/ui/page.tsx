'use client';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { useSystemLogController } from 'src/features/system-log-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';
import { SystemLogToolbar, AdminSystemLogsPanel } from 'src/widgets/admin-system-logs-panel';

export function SystemLogsPage() {
  const { t } = useTranslate('systemLog');
  const { t: tAdmin } = useTranslate('admin');
  const controller = useSystemLogController();
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
          onRefresh={controller.actions.refreshPage}
          refreshing={controller.resources.logs.isValidating}
          action={<SystemLogToolbar controller={controller} />}
        />
        <AdminSystemLogsPanel controller={controller} />
      </DashboardContent>
    </>
  );
}
