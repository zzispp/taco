'use client';

import { updateUserStatus } from 'src/features/user-management';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { UserToolbar } from './toolbar';
import { toggle, showError } from './helpers';
import { UserTableSection } from './table-section';
import { UserDialogSection } from './dialog-section';
import { useUserManagementController } from './controller';

export function UserManagementView() {
  const controller = useUserManagementController();
  const { resources, state, actions } = controller;
  const { t } = resources;

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.userManagement')}
        action={<Toolbar controller={controller} />}
      />
      <UserTableSection
        table={resources.table}
        filters={resources.filters}
        filterError={resources.filterError}
        users={resources.users}
        roles={resources.roles}
        depts={resources.depts}
        posts={resources.posts}
        deptTree={resources.deptTree}
        head={resources.head}
        loadingHead={resources.loadingHead}
        selectableUsers={resources.selectableUsers}
        selected={state.selected}
        canDelete={resources.canDelete}
        onFilterChange={resources.setFilters}
        onDeptSelect={actions.selectDept}
        onToggleAll={actions.toggleAll}
        onToggleSelected={(id) => state.setSelected(toggle(state.selected, id))}
        onEdit={actions.openEdit}
        onDelete={state.setDeleteTarget}
        onRoles={actions.openRoles}
        onResetPassword={actions.openPassword}
        onStatusChange={(user, status) =>
          updateUserStatus(user.user_id, status).catch(showError(t))
        }
      />
      <UserDialogSection {...controller} />
    </DashboardContent>
  );
}

type ToolbarProps = { controller: ReturnType<typeof useUserManagementController> };

function Toolbar({ controller }: ToolbarProps) {
  const { resources, state, actions } = controller;

  return (
    <UserToolbar
      t={resources.t}
      canAdd={resources.canAdd}
      canDelete={resources.canDelete}
      canImport={resources.canImport}
      canExport={resources.canExport}
      exportDisabled={resources.filterError !== null}
      selectedCount={state.selected.length}
      onCreate={actions.openCreate}
      onImport={() => state.setImportOpen(true)}
      onExport={actions.submitExport}
      onBatchDelete={() => state.setBatchDeleteOpen(true)}
    />
  );
}
