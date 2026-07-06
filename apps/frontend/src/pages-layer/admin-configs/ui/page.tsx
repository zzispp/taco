import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { ConfigManagementPanel } from 'src/widgets/admin-system-panels';

export function AdminConfigsPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.configManagement" />
      <ConfigManagementPanel />
    </>
  );
}
