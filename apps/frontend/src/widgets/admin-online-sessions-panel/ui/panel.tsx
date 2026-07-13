'use client';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { ForceLogoutDialog } from './confirm-dialog';
import { useOnlineSessionsController } from './controller';
import { OnlineSessionsTableSection } from './table-section';

export function OnlineSessionsPanel() {
  const { resources, state, actions } = useOnlineSessionsController();

  return (
    <DashboardContent>
      <AdminBreadcrumbs heading={resources.t('pages.onlineManagement')} />
      <OnlineSessionsTableSection
        table={resources.table}
        filters={resources.filters}
        rows={resources.rows}
        total={resources.sessions.total}
        head={resources.head}
        loading={resources.sessions.isLoading}
        canForceLogout={resources.canForceLogout}
        filterErrorMessage={resources.filterErrorMessage}
        onFilterChange={actions.setFilters}
        onForceLogout={state.setForceTarget}
      />
      <ForceLogoutDialog
        target={state.forceTarget}
        onClose={() => state.setForceTarget(null)}
        onConfirm={actions.confirmForceLogout}
      />
    </DashboardContent>
  );
}
