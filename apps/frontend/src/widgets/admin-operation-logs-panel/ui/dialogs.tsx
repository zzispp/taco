import type { OperationLogController } from 'src/features/audit-log-management';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { OperationLogDetailDialog } from 'src/features/audit-log-management';

export function OperationLogDialogs({ controller }: { controller: OperationLogController }) {
  return (
    <>
      <OperationLogDetailDialog controller={controller} />
      <OperationDeleteDialogs controller={controller} />
    </>
  );
}

function OperationDeleteDialogs({ controller }: { controller: OperationLogController }) {
  return (
    <>
      <SingleDeleteDialog controller={controller} />
      <BatchDeleteDialog controller={controller} />
      <CleanDialog controller={controller} />
    </>
  );
}

function SingleDeleteDialog({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      open={Boolean(state.deleteTarget)}
      onClose={() => state.setDeleteTarget(null)}
      title={t('actions.delete')}
      content={t('dialogs.deleteOne')}
      cancelText={t('cancel')}
      action={
        <Button
          color="error"
          variant="contained"
          loading={pending.has(`delete:${state.deleteTarget?.oper_id ?? ''}`)}
          onClick={actions.confirmDelete}
        >
          {t('actions.delete')}
        </Button>
      }
    />
  );
}

function BatchDeleteDialog({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      open={state.batchOpen}
      onClose={() => state.setBatchOpen(false)}
      title={t('actions.delete')}
      content={t('dialogs.deleteSelected', { count: state.table.selected.length })}
      cancelText={t('cancel')}
      action={
        <Button
          color="error"
          variant="contained"
          loading={pending.has('delete:batch')}
          onClick={actions.confirmBatchDelete}
        >
          {t('actions.delete')}
        </Button>
      }
    />
  );
}

function CleanDialog({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      open={state.cleanOpen}
      onClose={() => state.setCleanOpen(false)}
      title={t('actions.clean')}
      content={t('dialogs.cleanOperation')}
      cancelText={t('cancel')}
      action={
        <Button
          color="error"
          variant="contained"
          loading={pending.has('delete:clean')}
          onClick={actions.confirmClean}
        >
          {t('actions.clean')}
        </Button>
      }
    />
  );
}
