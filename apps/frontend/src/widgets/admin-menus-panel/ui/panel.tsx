'use client';

import { DashboardContent } from 'src/widgets/dashboard-shell';
import { AddButton, AdminBreadcrumbs } from 'src/widgets/admin-common';

import { MenuTableSection } from './table-section';
import { MenuDialogSection } from './dialog-section';
import { useMenuManagementController } from './controller';

export function MenuManagementView() {
  const controller = useMenuManagementController();
  const { resources, actions } = controller;
  const { t } = resources;

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.menuManagement')}
        action={
          resources.canAdd ? (
            <AddButton onClick={actions.openCreate}>{t('actions.addMenuItem')}</AddButton>
          ) : null
        }
      />
      <MenuTableSection {...controller} />
      <MenuDialogSection {...controller} />
    </DashboardContent>
  );
}
