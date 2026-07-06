import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { UserManagementPanel } from 'src/widgets/admin-users-panel';

export function AdminUsersPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.userManagement" />
      <UserManagementPanel />
    </>
  );
}
