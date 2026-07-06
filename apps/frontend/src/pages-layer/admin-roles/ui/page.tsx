import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { RoleManagementPanel } from 'src/widgets/admin-roles-panel';

export function AdminRolesPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.roleManagement" />
      <RoleManagementPanel />
    </>
  );
}
