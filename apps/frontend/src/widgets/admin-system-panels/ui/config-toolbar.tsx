import type {
  LocalDateTimeFilterError,
  LocalDateTimeFilterQuery,
} from 'src/shared/lib/local-date-time-filter';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

import { useHasPermission } from 'src/entities/session';

import { systemMutations } from 'src/features/system-management';

type ConfigToolbarProps = Readonly<{
  filters: LocalDateTimeFilterQuery;
  filterError: LocalDateTimeFilterError | null;
}>;

export function ConfigToolbar({ filters, filterError }: ConfigToolbarProps) {
  const { t } = useTranslate('admin');
  const canExport = useHasPermission('system:config:export');

  return (
    <Stack direction="row" spacing={1}>
      {canExport && <ConfigExportButton filters={filters} filterError={filterError} />}
      <Button
        variant="outlined"
        startIcon={<Iconify icon="solar:restart-bold" />}
        onClick={refreshCache(t)}
      >
        {t('actions.refreshCache')}
      </Button>
    </Stack>
  );
}

function ConfigExportButton({ filters, filterError }: ConfigToolbarProps) {
  const { t } = useTranslate('admin');

  return (
    <Button
      variant="outlined"
      disabled={filterError !== null}
      startIcon={<Iconify icon="solar:export-bold" />}
      onClick={() => exportItems({ filters, filterError, t })}
    >
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

async function exportItems({ filters, filterError, t }: ExportOptions) {
  if (filterError) {
    toast.error(t(LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY[filterError]));
    return;
  }
  try {
    await systemMutations.exportConfigs(filters);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
  }
}

type ExportOptions = ConfigToolbarProps & { t: ReturnType<typeof useTranslate>['t'] };
