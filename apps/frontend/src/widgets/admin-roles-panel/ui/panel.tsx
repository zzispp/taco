'use client';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { RoleToolbar } from './toolbar';
import { RoleTableSection } from './table-section';
import { RoleDialogSection } from './dialog-section';
import { useRoleManagementController } from './controller';

export function RoleManagementView() {
  const controller = useRoleManagementController();
  const { resources, dialogs, actions } = controller;
  const { t } = resources;

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.roleManagement')}
        action={
          <RoleToolbar
            t={t}
            canAdd={resources.canAdd}
            canDelete={resources.canDelete}
            canExport={resources.canExport}
            selectedCount={dialogs.selected.length}
            onCreate={actions.openCreate}
            onBatchDelete={() => dialogs.setBatchDeleteOpen(true)}
            onExport={actions.submitExport}
          />
        }
      />
      <RoleTableSection
        t={t}
        table={resources.table}
        filters={resources.filters}
        roles={resources.roles}
        head={resources.head}
        loadingHead={resources.loadingHead}
        selectableRoles={resources.selectableRoles}
        selected={dialogs.selected}
        canDelete={resources.canDelete}
        onFilterChange={resources.setFilters}
        onToggleAll={actions.toggleAll}
        onSelectedChange={dialogs.setSelected}
        onEdit={actions.openEdit}
        onDelete={dialogs.setDeleteTarget}
        onBind={actions.openBindings}
        onUsers={dialogs.setUsersTarget}
      />
      <RoleDialogSection {...controller} />
    </DashboardContent>
  );
}
