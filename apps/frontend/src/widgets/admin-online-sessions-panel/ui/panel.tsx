'use client';

import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

import { ForceLogoutDialog } from './confirm-dialog';
import { useOnlineSessionsController } from './controller';
import { OnlineSessionsTableSection } from './table-section';

export function OnlineSessionsPanel() {
  const { resources, state, actions } = useOnlineSessionsController();

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={resources.t('pages.onlineManagement')}
        onRefresh={actions.refreshPage}
        refreshing={resources.sessions.isValidating}
      />
      <OnlineSessionsTableSection
        table={resources.table}
        filters={resources.filters}
        sessions={resources.sessions}
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
