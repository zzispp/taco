import type { LoginLogController } from 'src/features/audit-log-management';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

export function LoginLogDialogs({ controller }: { controller: LoginLogController }) {
  return (
    <>
      <SingleDeleteDialog controller={controller} />
      <BatchDeleteDialog controller={controller} />
      <CleanDialog controller={controller} />
      <UnlockDialog controller={controller} />
    </>
  );
}

function SingleDeleteDialog({ controller }: { controller: LoginLogController }) {
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
          loading={pending.has(`delete:${state.deleteTarget?.info_id ?? ''}`)}
          onClick={actions.confirmDelete}
        >
          {t('actions.delete')}
        </Button>
      }
    />
  );
}

function BatchDeleteDialog({ controller }: { controller: LoginLogController }) {
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

function CleanDialog({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      open={state.cleanOpen}
      onClose={() => state.setCleanOpen(false)}
      title={t('actions.clean')}
      content={t('dialogs.cleanLogin')}
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

function UnlockDialog({ controller }: { controller: LoginLogController }) {
  const { t } = useTranslate('audit');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      open={Boolean(state.unlockTarget)}
      onClose={() => state.setUnlockTarget(null)}
      title={t('actions.unlock')}
      content={t('dialogs.unlock', { username: state.unlockTarget?.user_name ?? '' })}
      cancelText={t('cancel')}
      action={
        <Button
          color="warning"
          variant="contained"
          loading={pending.has('unlock')}
          onClick={actions.confirmUnlock}
        >
          {t('actions.unlock')}
        </Button>
      }
    />
  );
}
