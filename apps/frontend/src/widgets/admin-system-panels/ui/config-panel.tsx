'use client';

import type { ConfigItem, ConfigInput } from 'src/entities/system';

import { useMemo, useState } from 'react';

import { useTable } from 'src/shared/ui/table';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useConfigs } from 'src/entities/system';

import { systemMutations } from 'src/features/system-management';

import { SystemCrudPanel } from 'src/widgets/system-crud-panel';

import { ConfigToolbar } from './config-toolbar';
import { DEFAULT_CONFIG_INPUT, DEFAULT_CONFIG_FILTERS } from './config-constants';
import {
  configFields,
  toConfigFilters,
  configFilterFields,
  isConfigSelectable,
  normalizeConfigInput,
} from './config-fields';

export function ConfigManagementPanel() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_CONFIG_FILTERS);
  const resource = useConfigs(table.page, table.rowsPerPage, filters);
  const fields = useMemo(() => configFields(t), [t]);
  const filterFields = useMemo(() => configFilterFields(t), [t]);

  return (
    <SystemCrudPanel<ConfigItem, ConfigInput>
      isRowSelectable={isConfigSelectable}
      title={t('pages.configManagement')}
      addLabel={t('actions.addConfig')}
      idKey="config_id"
      nameKey="config_name"
      fields={fields}
      defaultInput={DEFAULT_CONFIG_INPUT}
      resource={resource}
      page={table.page}
      rowsPerPage={table.rowsPerPage}
      filters={filterFields}
      filterValues={filters}
      permissionPrefix="system:config"
      onFilterChange={(next) => setFilters(toConfigFilters(next))}
      onPageChange={table.onChangePage}
      onRowsPerPageChange={table.onChangeRowsPerPage}
      createItem={(input) => systemMutations.createConfig(normalizeConfigInput(input))}
      updateItem={(id, input) => systemMutations.updateConfig(id, normalizeConfigInput(input))}
      deleteItem={systemMutations.deleteConfig}
      batchDeleteItems={systemMutations.deleteConfigs}
      toolbarAction={<ConfigToolbar filters={filters} />}
    />
  );
}
