import type { DEFAULT_POST_FILTERS } from './post-constants';

import Button from '@mui/material/Button';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';

import { systemMutations } from 'src/features/system-management';

type PostToolbarProps = { filters: typeof DEFAULT_POST_FILTERS };

export function PostToolbar({ filters }: PostToolbarProps) {
  const { t } = useTranslate('admin');
  const canExport = useHasPermission('system:post:export');
  if (!canExport) return null;

  return (
    <Button variant="outlined" startIcon={<Iconify icon="solar:export-bold" />} onClick={exportItems(filters, t)}>
      {t('actions.export')}
    </Button>
  );
}

function exportItems(filters: typeof DEFAULT_POST_FILTERS, t: ReturnType<typeof useTranslate>['t']) {
  return async () => {
    try {
      await systemMutations.exportPosts(filters);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.exportFailed'));
    }
  };
}
