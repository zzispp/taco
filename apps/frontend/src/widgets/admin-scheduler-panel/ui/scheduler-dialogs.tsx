import type { SchedulerController } from 'src/features/scheduler-management';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { JobDialog, JobDetailDialog, ImportTaskDialog } from 'src/features/scheduler-management';

export function SchedulerDialogs({ controller }: { controller: SchedulerController }) {
  const { state, resources, actions } = controller;
  return (
    <>
      <ImportTaskDialog
        open={state.importOpen}
        tasks={resources.importable.data ?? []}
        loading={resources.importable.isLoading}
        error={resources.importable.error}
        onClose={() => state.setImportOpen(false)}
        onSelect={(task) => {
          state.setSelectedTask(task);
          state.setImportOpen(false);
        }}
      />
      <JobDialog
        open={Boolean(state.selectedTask || state.editing)}
        task={state.selectedTask}
        job={state.editing}
        onClose={actions.closeEditor}
        onSaved={actions.closeEditor}
      />
      <JobDetailDialog controller={controller} />
      <SchedulerDeleteDialogs controller={controller} />
    </>
  );
}

function SchedulerDeleteDialogs({ controller }: { controller: SchedulerController }) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const { state, actions, pending } = controller;
  const deleteButton = (batch: boolean) => (
    <Button
      color="error"
      variant="contained"
      loading={pending.has(batch ? 'delete:batch' : `delete:${state.deleteTarget?.job_id ?? ''}`)}
      onClick={batch ? actions.confirmBatchDelete : actions.confirmDelete}
    >
      {tAdmin('common.delete')}
    </Button>
  );
  return (
    <>
      <ConfirmDialog
        cancelText={tAdmin('common.cancel')}
        open={state.batchDeleteOpen}
        onClose={() => state.setBatchDeleteOpen(false)}
        title={tAdmin('common.delete')}
        content={t('confirmBatchDelete', { count: state.table.selected.length })}
        action={deleteButton(true)}
      />
      <ConfirmDialog
        cancelText={tAdmin('common.cancel')}
        open={Boolean(state.deleteTarget)}
        onClose={() => state.setDeleteTarget(null)}
        title={tAdmin('common.delete')}
        content={t('confirmDelete', { name: state.deleteTarget?.job_name ?? '' })}
        action={deleteButton(false)}
      />
    </>
  );
}
