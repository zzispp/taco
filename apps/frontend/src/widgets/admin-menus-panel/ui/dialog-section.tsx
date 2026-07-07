import type { MenuManagementController } from './controller';

import Button from '@mui/material/Button';

import { ConfirmDialog } from 'src/shared/ui/custom-dialog';

import { MenuDialog } from './dialog';

export function MenuDialogSection({ resources, dialogs, actions }: MenuManagementController) {
  const { t } = resources;

  return (
    <>
      <MenuDialog
        open={dialogs.creating || !!dialogs.editing}
        editing={!!dialogs.editing}
        submitting={dialogs.submitting}
        form={dialogs.form}
        menus={resources.allMenus.items}
        editingId={dialogs.editing?.menu_id}
        setForm={dialogs.setForm}
        onClose={actions.closeDialog}
        onSubmit={actions.submitMenu}
      />
      <ConfirmDialog
        open={!!dialogs.deleteTarget}
        onClose={() => dialogs.setDeleteTarget(null)}
        title={t('dialogs.deleteMenuItem')}
        content={t('dialogs.deleteContent', { name: dialogs.deleteTarget?.menu_name ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={actions.confirmDelete}>
            {t('common.delete')}
          </Button>
        }
      />
    </>
  );
}
