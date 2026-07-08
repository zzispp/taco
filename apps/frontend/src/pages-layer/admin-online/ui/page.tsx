import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { OnlineSessionsPanel } from 'src/widgets/admin-online-sessions-panel';

export function AdminOnlinePage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="pages.onlineManagement" />
      <OnlineSessionsPanel />
    </>
  );
}
