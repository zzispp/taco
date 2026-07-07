import type { DEFAULT_CONFIG_FILTERS } from './config-constants';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';

import { systemMutations } from 'src/features/system-management';

type ConfigToolbarProps = { filters: typeof DEFAULT_CONFIG_FILTERS };

export function ConfigToolbar({ filters }: ConfigToolbarProps) {
  const { t } = useTranslate('admin');
  const canExport = useHasPermission('system:config:export');

  return (
    <Stack direction="row" spacing={1}>
      {canExport && <ConfigExportButton filters={filters} />}
      <Button variant="outlined" startIcon={<Iconify icon="solar:restart-bold" />} onClick={refreshCache(t)}>
        {t('actions.refreshCache')}
      </Button>
    </Stack>
  );
}

function ConfigExportButton({ filters }: ConfigToolbarProps) {
  const { t } = useTranslate('admin');

  return (
    <Button variant="outlined" startIcon={<Iconify icon="solar:export-bold" />} onClick={exportItems(filters, t)}>
      {t('actions.export')}
    </Button>
  );
}

function refreshCache(t: ReturnType<typeof useTranslate>['t']) {
  return async () => {
    try {
      await systemMutations.refreshConfigCache();
      toast.success(t('messages.cacheRefreshed'));
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    }
  };
}

function exportItems(filters: typeof DEFAULT_CONFIG_FILTERS, t: ReturnType<typeof useTranslate>['t']) {
  return async () => {
    try {
      await systemMutations.exportConfigs(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  };
}
