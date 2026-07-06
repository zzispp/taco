import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { PostManagementPanel } from 'src/widgets/admin-system-panels';

export function AdminPostsPage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.postManagement" />
      <PostManagementPanel />
    </>
  );
}
