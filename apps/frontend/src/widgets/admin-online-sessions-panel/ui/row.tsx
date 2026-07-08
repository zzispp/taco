import type { OnlineSession } from 'src/entities/online-session';

import Box from '@mui/material/Box';
import Tooltip from '@mui/material/Tooltip';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { ONLINE_CELL_SX, formatLoginTime, ONLINE_ELLIPSIS_CELL_SX } from './helpers';

export function OnlineSessionRow({
  row,
  index,
  canForceLogout,
  onForceLogout,
}: {
  row: OnlineSession;
  index: number;
  canForceLogout: boolean;
  onForceLogout: (row: OnlineSession) => void;
}) {
  const { t } = useTranslate('admin');

  return (
    <TableRow hover>
      <TableCell sx={ONLINE_CELL_SX}>{index}</TableCell>
      <TableCell sx={ONLINE_ELLIPSIS_CELL_SX}>{row.tokenId}</TableCell>
      <TableCell sx={ONLINE_CELL_SX}>{row.userName}</TableCell>
      <TableCell sx={ONLINE_CELL_SX}>{row.deptName || '-'}</TableCell>
      <TableCell sx={ONLINE_CELL_SX}>{row.ipaddr}</TableCell>
      <TableCell sx={ONLINE_ELLIPSIS_CELL_SX}>{row.loginLocation}</TableCell>
      <TableCell sx={ONLINE_CELL_SX}>{row.browser}</TableCell>
      <TableCell sx={ONLINE_CELL_SX}>{row.os}</TableCell>
      <TableCell sx={ONLINE_CELL_SX}>{formatLoginTime(row.loginTime)}</TableCell>
      <TableCell align="right" sx={ONLINE_CELL_SX}>
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Tooltip title={t('onlineSessions.forceLogout')}>
            <span>
              <IconButton
                color="error"
                disabled={!canForceLogout}
                onClick={() => onForceLogout(row)}
              >
                <Iconify icon="solar:trash-bin-trash-bold" />
              </IconButton>
            </span>
          </Tooltip>
        </Box>
      </TableCell>
    </TableRow>
  );
}
