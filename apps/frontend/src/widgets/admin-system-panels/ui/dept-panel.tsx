'use client';

import { DashboardContent } from 'src/widgets/dashboard-shell';
import { AddButton, AdminBreadcrumbs } from 'src/widgets/admin-common';

import { DeptTableSection } from './dept-table-section';
import { DeptDialogSection } from './dept-dialog-section';
import { useDeptManagementController } from './dept-controller';

export function DeptManagementPanel() {
  const controller = useDeptManagementController();
  const { resources, actions } = controller;
  const { t } = resources;

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.deptManagement')}
        action={
          resources.canAdd ? (
            <AddButton onClick={actions.openCreate}>{t('actions.addDept')}</AddButton>
          ) : null
        }
      />
      <DeptTableSection {...controller} />
      <DeptDialogSection {...controller} />
    </DashboardContent>
  );
}
