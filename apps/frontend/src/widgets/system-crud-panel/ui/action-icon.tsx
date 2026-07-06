import type { ActionIconProps } from './types';

import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';

export function ActionIcon({ title, icon, disabled, color, onClick }: ActionIconProps) {
  return (
    <Tooltip title={title}>
      <span>
        <IconButton color={color} disabled={disabled} onClick={onClick}>
          <Iconify icon={icon} />
        </IconButton>
      </span>
    </Tooltip>
  );
}
