import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { DeptManagementPanel } from 'src/widgets/admin-system-panels';

export function AdminDeptsPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.deptManagement" />
      <DeptManagementPanel />
    </>
  );
}
