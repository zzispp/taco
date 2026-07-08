import type { TranslateFn } from 'src/shared/i18n';
import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { OnlineSession } from 'src/entities/online-session';

import dayjs from 'dayjs';

export const ONLINE_CELL_SX = { whiteSpace: 'nowrap' } as const;
export const ONLINE_ELLIPSIS_CELL_SX = {
  whiteSpace: 'nowrap',
  maxWidth: 240,
  overflow: 'hidden',
  textOverflow: 'ellipsis',
} as const;

const ONLINE_HEAD_SX = { whiteSpace: 'nowrap' } as const;

export function onlineSessionHead(t: TranslateFn): TableHeadCellProps[] {
  return [
    { id: 'index', label: t('common.index'), width: 80, sx: ONLINE_HEAD_SX },
    { id: 'tokenId', label: t('onlineSessions.tokenId'), width: 260, sx: ONLINE_HEAD_SX },
    { id: 'userName', label: t('onlineSessions.userName'), width: 140, sx: ONLINE_HEAD_SX },
    { id: 'deptName', label: t('fields.deptName'), width: 140, sx: ONLINE_HEAD_SX },
    { id: 'ipaddr', label: t('onlineSessions.host'), width: 150, sx: ONLINE_HEAD_SX },
    { id: 'loginLocation', label: t('onlineSessions.loginLocation'), width: 180, sx: ONLINE_HEAD_SX },
    { id: 'browser', label: t('onlineSessions.browser'), width: 120, sx: ONLINE_HEAD_SX },
    { id: 'os', label: t('onlineSessions.os'), width: 120, sx: ONLINE_HEAD_SX },
    { id: 'loginTime', label: t('onlineSessions.loginTime'), width: 190, sx: ONLINE_HEAD_SX },
    { id: 'actions', label: t('common.actions'), align: 'right', width: 110, sx: ONLINE_HEAD_SX },
  ];
}

export function pageRows(rows: OnlineSession[], page: number, rowsPerPage: number) {
  const start = page * rowsPerPage;
  return rows.slice(start, start + rowsPerPage);
}

export function formatLoginTime(value: number) {
  const date = dayjs(value);
  return date.isValid() ? date.format('YYYY-MM-DD HH:mm:ss') : 'Invalid';
}
