'use client';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { DictHeaderActions } from './dict-toolbar';
import { DictDialogSection } from './dict-dialog-section';
import { DictPanelSections } from './dict-panel-sections';
import { useDictManagementController } from './dict-controller';

export function DictManagementPanel() {
  const controller = useDictManagementController();
  const { resources, state, actions } = controller;
  const { t } = resources;

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.dictManagement')}
        action={
          <DictHeaderActions
            t={t}
            canAdd={resources.canAdd}
            canExport={resources.canExport}
            canRefresh={resources.canRemove}
            canRemove={resources.canRemove}
            selectedCount={state.selectedTypeIds.length}
            onAdd={() => state.setCreatingType(true)}
            onExport={actions.exportTypes}
            onRefresh={actions.refreshCache}
            onBatchDelete={() => state.setBatchDeleteTypeOpen(true)}
          />
        }
      />
      <DictPanelSections {...controller} />
      <DictDialogSection {...controller} />
    </DashboardContent>
  );
}
