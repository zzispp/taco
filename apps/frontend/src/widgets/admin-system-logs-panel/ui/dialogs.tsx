import type { SystemLogFilterQuery } from 'src/entities/system-log';
import type { SystemLogController } from 'src/features/system-log-management';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { SystemLogDetailDialog } from 'src/features/system-log-management';

export function SystemLogDialogs({ controller }: { controller: SystemLogController }) {
  return (
    <>
      <SystemLogDetailDialog controller={controller} />
      <DeleteDialogs controller={controller} />
      <CleanupDialog controller={controller} />
      <CleanupExecutionDialog controller={controller} />
    </>
  );
}

function CleanupExecutionDialog({ controller }: { controller: SystemLogController }) {
  const { t } = useTranslate('systemLog');
  const { state, resources, actions } = controller;
  const execution = resources.cleanupExecution.data;
  const content = resources.cleanupExecution.error
    ? t('messages.cleanExecutionStatusFailure')
    : t(`cleanupStates.${execution?.state ?? 'pending'}`, {
        count: execution?.deleted ?? 0,
        batches: execution?.batches ?? 0,
      });
  return (
    <ConfirmDialog
      open={state.cleanupExecutionDialogOpen}
      onClose={actions.dismissCleanupExecution}
      title={t('actions.clean')}
      content={content}
      action={null}
      cancelText={t('close')}
    />
  );
}

function DeleteDialogs({ controller }: { controller: SystemLogController }) {
  const { t } = useTranslate('systemLog');
  const { state, actions, pending } = controller;
  return (
    <>
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
            loading={pending.has(`delete:${state.deleteTarget?.log_id ?? ''}`)}
            onClick={actions.confirmDelete}
          >
            {t('actions.delete')}
          </Button>
        }
      />
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
    </>
  );
}

function CleanupDialog({ controller }: { controller: SystemLogController }) {
  const { t } = useTranslate('systemLog');
  const { state, actions, pending } = controller;
  return (
    <ConfirmDialog
      open={state.cleanPreview !== null}
      onClose={actions.cancelClean}
      title={t('actions.clean')}
      content={
        <CleanupConfirmation
          count={state.cleanPreview?.count ?? 0}
          query={state.cleanPreview?.query}
        />
      }
      cancelText={t('cancel')}
      action={
        <Button
          color="error"
          variant="contained"
          loading={pending.has('clean:confirm')}
          onClick={actions.confirmClean}
        >
          {t('actions.clean')}
        </Button>
      }
    />
  );
}

function CleanupConfirmation({
  count,
  query,
}: {
  count: number;
  query: SystemLogFilterQuery | undefined;
}) {
  const { t } = useTranslate('systemLog');
  if (!query) return null;
  return (
    <Box>
      <Typography>{t('dialogs.cleanMatched', { count })}</Typography>
      <Typography variant="subtitle2" sx={{ mt: 2 }}>
        {t('dialogs.cleanFilterSnapshot')}
      </Typography>
      <Typography variant="body2">{formatCleanupFilters(query, t)}</Typography>
    </Box>
  );
}

function formatCleanupFilters(query: SystemLogFilterQuery, t: (key: string) => string) {
  const values = [
    query.keyword && `${t('fields.keyword')}: ${query.keyword}`,
    query.levels && `${t('fields.level')}: ${query.levels}`,
    query.target && `${t('fields.target')}: ${query.target}`,
    query.begin_time && `${t('fields.beginTime')}: ${fAdminDateTime(query.begin_time)}`,
    query.end_time && `${t('fields.endTime')}: ${fAdminDateTime(query.end_time)}`,
  ];
  return values.filter(Boolean).join('; ');
}
