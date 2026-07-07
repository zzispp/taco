import { LocalizedDashboardDocumentTitle } from 'src/shared/i18n';

import { DashboardContent } from 'src/widgets/dashboard-shell';
import { SystemDashboardPanel } from 'src/widgets/system-dashboard-panel';

export function DashboardHomePage() {
  return (
    <>
      <LocalizedDashboardDocumentTitle titleKey="nav.dashboard" />
      <DashboardContent maxWidth="xl">
        <SystemDashboardPanel />
      </DashboardContent>
    </>
  );
}
