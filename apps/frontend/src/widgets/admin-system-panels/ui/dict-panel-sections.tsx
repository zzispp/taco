import type { DictManagementController } from './dict-controller';

import Stack from '@mui/material/Stack';

import { toggle } from './dict-helpers';
import { DictDataSection, DictTypeSection } from './dict-sections';

export function DictPanelSections({ resources, state, actions }: DictManagementController) {
  return (
    <Stack spacing={3}>
      <DictTypePanelSection resources={resources} state={state} actions={actions} />
      <DictDataPanelSection resources={resources} state={state} actions={actions} />
    </Stack>
  );
}

type DictPanelSectionProps = DictManagementController;

function DictTypePanelSection({ resources, state, actions }: DictPanelSectionProps) {
  return (
    <DictTypeSection
      table={resources.typeTable}
      filters={resources.typeFilters}
      resource={resources.dictTypes}
      activeType={resources.activeType}
      head={resources.typeHead}
      loadingHead={resources.loadingTypeHead}
      selectedIds={state.selectedTypeIds}
      canRemove={resources.canRemove}
      onFilterChange={resources.setTypeFilters}
      onToggleAll={actions.toggleAllTypes}
      onToggleRow={(id) => state.setSelectedTypeIds(toggle(state.selectedTypeIds, id))}
      onSelect={state.setSelected}
      onEdit={actions.openTypeEdit}
      onDelete={state.setDeleteType}
    />
  );
}

function DictDataPanelSection({ resources, state, actions }: DictPanelSectionProps) {
  return (
    <DictDataSection
      table={resources.dataTable}
      filters={resources.dataFilters}
      resource={resources.dictData}
      activeType={resources.activeType}
      head={resources.dataHead}
      loadingHead={resources.loadingDataHead}
      selectedIds={state.selectedDataIds}
      canAdd={resources.canAdd}
      canRemove={resources.canRemove}
      canExport={resources.canExport}
      onFilterChange={resources.setDataFilters}
      onToggleAll={actions.toggleAllData}
      onToggleRow={(id) => state.setSelectedDataIds(toggle(state.selectedDataIds, id))}
      onEdit={actions.openDataEdit}
      onDelete={state.setDeleteData}
      onAdd={actions.openCreateData}
      onBatchDelete={() => state.setBatchDeleteDataOpen(true)}
      onExport={actions.exportData}
    />
  );
}
