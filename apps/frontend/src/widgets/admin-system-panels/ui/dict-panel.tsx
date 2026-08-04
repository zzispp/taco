'use client';

import { AdminBreadcrumbs } from 'src/shared/ui/admin-common';
import { DashboardContent } from 'src/shared/ui/dashboard-content';

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
        onRefresh={actions.refreshPage}
        refreshing={resources.dictTypes.isValidating || resources.dictData.isValidating}
        action={
          <DictHeaderActions
            t={t}
            canAdd={resources.canAdd}
            canExport={resources.canExport}
            exportDisabled={resources.typeFilterError !== null}
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
