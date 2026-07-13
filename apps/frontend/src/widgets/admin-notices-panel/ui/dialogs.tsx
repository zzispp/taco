import type { NoticeManagementController } from 'src/features/notice-management';

import Button from '@mui/material/Button';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { NoticeDetailDrawer } from 'src/entities/notice';

import { NoticeFormDialog } from 'src/features/notice-management';

import { NoticeReaderDialog } from './reader-dialog';

export function NoticeDialogs({ controller }: { controller: NoticeManagementController }) {
  const { state, resources, permissions, actions, pending } = controller;
  return (
    <>
      <NoticeFormDialog
        open={state.creating || Boolean(state.editingId)}
        notice={resources.editor.data}
        editing={Boolean(state.editingId)}
        loading={Boolean(state.editingId) && resources.editor.isLoading}
        error={state.editingId ? resources.editor.error : undefined}
        submitting={pending === 'create' || pending?.startsWith('edit:') === true}
        onClose={actions.closeEditor}
        onSubmit={actions.submit}
      />
      <NoticeDetailDrawer
        open={Boolean(state.detailId) && permissions.canOpenDetail}
        notice={resources.detail.data}
        loading={resources.detail.isLoading}
        error={resources.detail.error}
        onClose={() => state.setDetailId(null)}
      />
      <NoticeReaderDialog controller={controller} />
      <NoticeDeleteDialogs controller={controller} />
    </>
  );
}

function NoticeDeleteDialogs({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state, actions, pending } = controller;
  return (
    <>
      <ConfirmDialog
        open={Boolean(state.deleteTarget)}
        title={t('common.delete')}
        content={t('dialogs.deleteContent', { name: state.deleteTarget?.notice_title ?? '' })}
        cancelText={t('common.cancel')}
        onClose={() => state.setDeleteTarget(null)}
        action={
          <Button
            color="error"
            variant="contained"
            loading={pending?.startsWith('delete:') === true}
            onClick={actions.confirmDelete}
          >
            {t('common.delete')}
          </Button>
        }
      />
      <ConfirmDialog
        open={state.batchDeleteOpen}
        title={t('common.delete')}
        content={t('notice.batchDeleteContent', { count: state.table.selected.length })}
        cancelText={t('common.cancel')}
        onClose={() => state.setBatchDeleteOpen(false)}
        action={
          <Button
            color="error"
            variant="contained"
            loading={pending === 'delete:batch'}
            onClick={actions.confirmBatchDelete}
          >
            {t('common.delete')}
          </Button>
        }
      />
    </>
  );
}
