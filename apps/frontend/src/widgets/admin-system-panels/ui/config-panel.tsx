'use client';

import type { ConfigItem, ConfigInput } from 'src/entities/system';

import { useMemo, useState } from 'react';

import Button from '@mui/material/Button';

import { useTable } from 'src/shared/ui/table';
import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useConfigs } from 'src/entities/system';

import { systemMutations } from 'src/features/system-management/api';

import { SystemCrudPanel } from 'src/widgets/system-crud-panel';

const DEFAULT_INPUT: ConfigInput = { config_name: '', config_key: '', config_value: '', config_type: 'N', remark: '' };
const DEFAULT_FILTERS = { config_name: '', config_key: '', config_type: '', begin_time: '', end_time: '' };

export function ConfigManagementPanel() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const resource = useConfigs(table.page, table.rowsPerPage, filters);
  const fields = useMemo(() => [
    { key: 'config_name' as const, label: t('fields.configName') },
    { key: 'config_key' as const, label: t('fields.configKey') },
    { key: 'config_value' as const, label: t('fields.configValue') },
    { key: 'config_type' as const, label: t('fields.configType'), type: 'select' as const, options: configTypeOptions(t) },
    { key: 'remark' as const, label: t('common.remark'), type: 'textarea' as const },
    { key: 'create_time' as const, label: t('fields.createTime'), format: 'dateTime' as const, hiddenInForm: true },
  ], [t]);
  const filterFields = useMemo(() => [
    { key: 'config_name', label: t('fields.configName') },
    { key: 'config_key', label: t('fields.configKey') },
    { key: 'config_type', label: t('fields.configType'), type: 'select' as const, options: allConfigTypeOptions(t) },
    { key: 'begin_time', label: t('fields.beginTime'), type: 'date' as const },
    { key: 'end_time', label: t('fields.endTime'), type: 'date' as const },
  ], [t]);

  const refreshCache = async () => {
    try {
      await systemMutations.refreshConfigCache();
      toast.success(t('messages.cacheRefreshed'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  };

  return <SystemCrudPanel<ConfigItem, ConfigInput> title={t('pages.configManagement')} addLabel={t('actions.addConfig')} idKey="config_id" nameKey="config_name" fields={fields} defaultInput={DEFAULT_INPUT} resource={resource} page={table.page} rowsPerPage={table.rowsPerPage} filters={filterFields} filterValues={filters} permissionPrefix="system:config" onFilterChange={(next) => setFilters(toConfigFilters(next))} onPageChange={table.onChangePage} onRowsPerPageChange={table.onChangeRowsPerPage} createItem={systemMutations.createConfig} updateItem={systemMutations.updateConfig} deleteItem={systemMutations.deleteConfig} batchDeleteItems={systemMutations.deleteConfigs} toolbarAction={<Button variant="outlined" startIcon={<Iconify icon="solar:restart-bold" />} onClick={refreshCache}>{t('actions.refreshCache')}</Button>} />;
}

function configTypeOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [{ value: 'Y', label: t('common.yes') }, { value: 'N', label: t('common.no') }];
}

function allConfigTypeOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [{ value: '', label: t('common.all') }, ...configTypeOptions(t)];
}

function toConfigFilters(values: Record<string, string>) {
  return { config_name: values.config_name ?? '', config_key: values.config_key ?? '', config_type: values.config_type ?? '', begin_time: values.begin_time ?? '', end_time: values.end_time ?? '' };
}
