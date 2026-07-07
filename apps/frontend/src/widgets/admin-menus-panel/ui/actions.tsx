import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

type MenuActionsProps = {
  canAdd: boolean;
  canEdit: boolean;
  canDelete: boolean;
  onCreateChild: () => void;
  onEdit: () => void;
  onDelete: () => void;
};

export function MenuActions({
  canAdd,
  canEdit,
  canDelete,
  onCreateChild,
  onEdit,
  onDelete,
}: MenuActionsProps) {
  const { t } = useTranslate('admin');

  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      <Tooltip title={t('common.add')}>
        <span>
          <IconButton disabled={!canAdd} onClick={onCreateChild}>
            <Iconify icon="mingcute:add-line" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={!canEdit} onClick={onEdit}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={!canDelete} onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}
