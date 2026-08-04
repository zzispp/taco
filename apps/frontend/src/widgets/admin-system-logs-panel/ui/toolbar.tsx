import type { SystemLogController } from 'src/features/system-log-management';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

type SystemLogToolbarProps = { controller: SystemLogController };

export function SystemLogToolbar({ controller }: SystemLogToolbarProps) {
  return (
    <Stack direction="row" useFlexGap flexWrap="wrap" spacing={1}>
      <BatchDeleteAction controller={controller} />
      <CleanupAction controller={controller} />
      <CleanupStatusAction controller={controller} />
      <ExportAction controller={controller} />
    </Stack>
  );
}

function BatchDeleteAction({ controller }: SystemLogToolbarProps) {
  const { t } = useTranslate('systemLog');
  const { state, resources, pending } = controller;
  if (!resources.canRemove) return null;

  return (
    <Button
      color="error"
      variant="outlined"
      disabled={state.table.selected.length === 0 || pending.has('delete:batch')}
      loading={pending.has('delete:batch')}
      startIcon={<Iconify icon="solar:trash-bin-trash-bold" />}
      onClick={() => state.setBatchOpen(true)}
    >
      {t('actions.delete')}
    </Button>
  );
}

function CleanupAction({ controller }: SystemLogToolbarProps) {
  const { t } = useTranslate('systemLog');
  const { state, resources, pending, actions } = controller;
  if (!resources.canRemove) return null;

  return (
    <Button
      color="error"
      variant="outlined"
      disabled={
        Boolean(state.cleanupExecutionId) ||
        pending.has('clean:count') ||
        pending.has('clean:confirm')
      }
      loading={pending.has('clean:count')}
      startIcon={<Iconify icon="solar:eraser-bold" />}
      onClick={actions.requestClean}
    >
      {t('actions.clean')}
    </Button>
  );
}

function CleanupStatusAction({ controller }: SystemLogToolbarProps) {
  const { t } = useTranslate('systemLog');
  const { state, actions } = controller;
  if (!state.cleanupExecutionId) return null;

  return (
    <Tooltip title={t('actions.cleanupStatus')}>
      <IconButton aria-label={t('actions.cleanupStatus')} onClick={actions.showCleanupExecution}>
        <Iconify icon="solar:clock-circle-bold" />
      </IconButton>
    </Tooltip>
  );
}

function ExportAction({ controller }: SystemLogToolbarProps) {
  const { t } = useTranslate('systemLog');
  const { resources, pending, actions } = controller;
  if (!resources.canExport) return null;

  return (
    <Button
      variant="outlined"
      disabled={!resources.filtersValid || !resources.hasRequiredRange || pending.has('export')}
      loading={pending.has('export')}
      startIcon={<Iconify icon="solar:export-bold" />}
      onClick={actions.submitExport}
    >
      {t('actions.export')}
    </Button>
  );
}
