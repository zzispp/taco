import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { MenuManagementPanel } from 'src/widgets/admin-menus-panel';

export function AdminMenusPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.menuManagement" />
      <MenuManagementPanel />
    </>
  );
}
