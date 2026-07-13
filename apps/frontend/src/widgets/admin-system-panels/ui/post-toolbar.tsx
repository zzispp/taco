import type {
  LocalDateTimeFilterError,
  LocalDateTimeFilterQuery,
} from 'src/shared/lib/local-date-time-filter';

import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

import { useHasPermission } from 'src/entities/session';

import { systemMutations } from 'src/features/system-management';

type PostToolbarProps = Readonly<{
  filters: LocalDateTimeFilterQuery;
  filterError: LocalDateTimeFilterError | null;
}>;

export function PostToolbar({ filters, filterError }: PostToolbarProps) {
  const { t } = useTranslate('admin');
  const canExport = useHasPermission('system:post:export');
  if (!canExport) return null;

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

async function exportItems({ filters, filterError, t }: ExportOptions) {
  if (filterError) {
    toast.error(t(LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY[filterError]));
    return;
  }
  try {
    await systemMutations.exportPosts(filters);
  } catch (error) {
    toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
  }
}

type ExportOptions = PostToolbarProps & { t: ReturnType<typeof useTranslate>['t'] };
