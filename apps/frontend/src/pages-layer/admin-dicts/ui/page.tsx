import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { DictManagementPanel } from 'src/widgets/admin-system-panels';

export function AdminDictsPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.dictManagement" />
      <DictManagementPanel />
    </>
  );
}
