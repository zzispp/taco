'use client';

import type { ConfigItem, ConfigInput } from 'src/entities/system';

import { useMemo, useState } from 'react';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { useTable } from 'src/shared/ui/table';
import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useConfigs } from 'src/entities/system';
import { useHasPermission } from 'src/entities/session';

import { systemMutations } from 'src/features/system-management/api';

import { SystemCrudPanel } from 'src/widgets/system-crud-panel';

const DEFAULT_INPUT: ConfigInput = {
  config_name: '',
  config_key: '',
  config_value: '',
  config_type: 'N',
  public_read: false,
  remark: '',
};
const DEFAULT_FILTERS = {
  config_name: '',
  config_key: '',
  config_type: '',
  begin_time: '',
  end_time: '',
};

export function ConfigManagementPanel() {
  const { t } = useTranslate('admin');
  const table = useTable({ defaultRowsPerPage: 10 });
  const [filters, setFilters] = useState(DEFAULT_FILTERS);
  const resource = useConfigs(table.page, table.rowsPerPage, filters);
  const canExport = useHasPermission('system:config:export');
  const fields = useMemo(
    () => [
      { key: 'config_name' as const, label: t('fields.configName') },
      { key: 'config_key' as const, label: t('fields.configKey'), disabled: builtInFieldDisabled },
      { key: 'config_value' as const, label: t('fields.configValue') },
      {
        key: 'config_type' as const,
        label: t('fields.configType'),
        type: 'select' as const,
        options: configTypeOptions(t),
        disabled: builtInFieldDisabled,
      },
      {
        key: 'public_read' as const,
        label: t('fields.publicRead'),
        type: 'boolean' as const,
        disabled: publicReadDisabled,
      },
      { key: 'remark' as const, label: t('common.remark'), type: 'textarea' as const },
      {
        key: 'create_time' as const,
        label: t('fields.createTime'),
        format: 'dateTime' as const,
        hiddenInForm: true,
      },
    ],
    [t]
  );
  const filterFields = useMemo(
    () => [
      { key: 'config_name', label: t('fields.configName') },
      { key: 'config_key', label: t('fields.configKey') },
      {
        key: 'config_type',
        label: t('fields.configType'),
        type: 'select' as const,
        options: allConfigTypeOptions(t),
      },
      { key: 'begin_time', label: t('fields.beginTime'), type: 'date' as const },
      { key: 'end_time', label: t('fields.endTime'), type: 'date' as const },
    ],
    [t]
  );

  const refreshCache = async () => {
    try {
      await systemMutations.refreshConfigCache();
      toast.success(t('messages.cacheRefreshed'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  };

  const exportItems = async () => {
    try {
      await systemMutations.exportConfigs(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  };

  return (
    <SystemCrudPanel<ConfigItem, ConfigInput>
      isRowSelectable={isConfigSelectable}
      title={t('pages.configManagement')}
      addLabel={t('actions.addConfig')}
      idKey="config_id"
      nameKey="config_name"
      fields={fields}
      defaultInput={DEFAULT_INPUT}
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
      toolbarAction={
        <Stack direction="row" spacing={1}>
          {canExport && (
            <Button
              variant="outlined"
              startIcon={<Iconify icon="solar:export-bold" />}
              onClick={exportItems}
            >
              {t('actions.export')}
            </Button>
          )}
          <Button
            variant="outlined"
            startIcon={<Iconify icon="solar:restart-bold" />}
            onClick={refreshCache}
          >
            {t('actions.refreshCache')}
          </Button>
        </Stack>
      }
    />
  );
}

function configTypeOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [
    { value: 'Y', label: t('common.yes') },
    { value: 'N', label: t('common.no') },
  ];
}

function allConfigTypeOptions(t: ReturnType<typeof useTranslate>['t']) {
  return [{ value: '', label: t('common.all') }, ...configTypeOptions(t)];
}

function toConfigFilters(values: Record<string, string>) {
  return {
    config_name: values.config_name ?? '',
    config_key: values.config_key ?? '',
    config_type: values.config_type ?? '',
    begin_time: values.begin_time ?? '',
    end_time: values.end_time ?? '',
  };
}

function isConfigSelectable(row: ConfigItem) {
  return row.config_type !== 'Y';
}

function builtInFieldDisabled({ editing }: { editing: Record<string, unknown> | null }) {
  return editing?.config_type === 'Y';
}

function publicReadDisabled({
  form,
  editing,
}: {
  form: Record<string, unknown>;
  editing: Record<string, unknown> | null;
}) {
  const key = String(form.config_key || editing?.config_key || '');
  return key === 'sys.user.initPassword';
}

function normalizeConfigInput(input: ConfigInput): ConfigInput {
  if (input.config_key === 'sys.user.initPassword') {
    return { ...input, public_read: false };
  }
  return input;
}
