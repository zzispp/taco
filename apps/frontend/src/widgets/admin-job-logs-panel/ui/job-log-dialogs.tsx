import type { JobLogController } from 'src/features/scheduler-management';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { ExecutionDetailDialog } from 'src/features/scheduler-management';

export function JobLogDialogs({ controller }: { controller: JobLogController }) {
  return (
    <>
      <ExecutionDetailDialog controller={controller} />
      <BatchAndCleanDialogs controller={controller} />
      <SingleLogDeleteDialog controller={controller} />
    </>
  );
}

function BatchAndCleanDialogs({ controller }: { controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { state, actions, pending } = controller;
  return (
    <>
      <ConfirmDialog
        cancelText={tAdmin('common.cancel')}
        open={state.batchOpen}
        onClose={() => state.setBatchOpen(false)}
        title={tAdmin('common.delete')}
        content={tAdmin('dialogs.deleteContent', { name: String(state.table.selected.length) })}
        action={
          <Button
            color="error"
            variant="contained"
            loading={pending.has('delete:batch')}
            onClick={actions.confirmBatchDelete}
          >
            {tAdmin('common.delete')}
          </Button>
        }
      />
      <ConfirmDialog
        cancelText={tAdmin('common.cancel')}
        open={state.cleanOpen}
        onClose={() => state.setCleanOpen(false)}
        title={t('clearLogs')}
        content={t('clearLogsGlobalConfirm')}
        action={
          <Button
            color="error"
            variant="contained"
            loading={pending.has('delete:clean')}
            onClick={actions.confirmClean}
          >
            {t('clearLogs')}
          </Button>
        }
      />
    </>
  );
}

function SingleLogDeleteDialog({ controller }: { controller: JobLogController }) {
  const { t: tAdmin } = useTranslate('admin');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      cancelText={tAdmin('common.cancel')}
      open={Boolean(state.deleteTarget)}
      onClose={() => state.setDeleteTarget(null)}
      title={tAdmin('common.delete')}
      content={tAdmin('dialogs.deleteContent', { name: state.deleteTarget?.job_name ?? '' })}
      action={
        <Button
          color="error"
          variant="contained"
          loading={pending.has(`delete:${state.deleteTarget?.execution_id ?? ''}`)}
          onClick={actions.confirmDelete}
        >
          {tAdmin('common.delete')}
        </Button>
      }
    />
  );
}
