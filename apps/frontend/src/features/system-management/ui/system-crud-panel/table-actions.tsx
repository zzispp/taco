import type React from 'react';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';

export function TableActions({
  permissionPrefix,
  extra,
  deleteDisabled,
  onEdit,
  onDelete,
}: {
  permissionPrefix: string;
  extra?: React.ReactNode;
  deleteDisabled?: boolean;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslate('admin');
  const canEdit = useHasPermission(`${permissionPrefix}:edit`);
  const canDelete = useHasPermission(`${permissionPrefix}:remove`);
  return (
    <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
      {extra}
      <Tooltip title={t('common.edit')}>
        <span>
          <IconButton disabled={!canEdit} onClick={onEdit}>
            <Iconify icon="solar:pen-bold" />
          </IconButton>
        </span>
      </Tooltip>
      <Tooltip title={t('common.delete')}>
        <span>
          <IconButton color="error" disabled={!canDelete || deleteDisabled} onClick={onDelete}>
            <Iconify icon="solar:trash-bin-trash-bold" />
          </IconButton>
        </span>
      </Tooltip>
    </Box>
  );
}
