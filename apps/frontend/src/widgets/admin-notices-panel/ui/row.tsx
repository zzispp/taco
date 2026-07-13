import type { NoticeSummary } from 'src/entities/notice';
import type { IconifyName } from 'src/shared/ui/iconify';
import type { NoticeManagementController } from 'src/features/notice-management';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import Checkbox from '@mui/material/Checkbox';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  noticeTypeColors,
  noticeStatusColors,
  noticeTypeTranslationKeys,
  noticeStatusTranslationKeys,
} from 'src/entities/notice';

export function NoticeRow({
  notice,
  controller,
}: {
  notice: NoticeSummary;
  controller: NoticeManagementController;
}) {
  return (
    <TableRow hover>
      {controller.permissions.canRemove ? (
        <NoticeSelectionCell notice={notice} controller={controller} />
      ) : null}
      <NoticeTitleCell notice={notice} controller={controller} />
      <NoticeTypeCell notice={notice} />
      <NoticeStatusCell notice={notice} />
      <TableCell>{notice.create_by}</TableCell>
      <TableCell>{fAdminDateTime(notice.create_time)}</TableCell>
      <NoticeActions notice={notice} controller={controller} />
    </TableRow>
  );
}

function NoticeSelectionCell({
  notice,
  controller,
}: {
  notice: NoticeSummary;
  controller: NoticeManagementController;
}) {
  return (
    <TableCell padding="checkbox">
      <Checkbox
        checked={controller.state.table.selected.includes(notice.notice_id)}
        onChange={() => controller.state.table.onSelectRow(notice.notice_id)}
      />
    </TableCell>
  );
}

function NoticeTitleCell({
  notice,
  controller,
}: {
  notice: NoticeSummary;
  controller: NoticeManagementController;
}) {
  return (
    <TableCell sx={{ maxWidth: 300 }}>
      <Button
        variant="text"
        color="inherit"
        disabled={!controller.permissions.canOpenDetail}
        onClick={() => controller.state.setDetailId(notice.notice_id)}
        sx={{
          maxWidth: '100%',
          justifyContent: 'flex-start',
          textAlign: 'left',
          textTransform: 'none',
        }}
      >
        {notice.notice_title}
      </Button>
    </TableCell>
  );
}

function NoticeTypeCell({ notice }: { notice: NoticeSummary }) {
  const { t } = useTranslate('admin');
  return (
    <TableCell>
      <Label color={noticeTypeColors[notice.notice_type]} variant="soft">
        {t(noticeTypeTranslationKeys[notice.notice_type])}
      </Label>
    </TableCell>
  );
}

function NoticeStatusCell({ notice }: { notice: NoticeSummary }) {
  const { t } = useTranslate('admin');
  return (
    <TableCell>
      <Label color={noticeStatusColors[notice.status]} variant="soft">
        {t(noticeStatusTranslationKeys[notice.status])}
      </Label>
    </TableCell>
  );
}

function NoticeActions({
  notice,
  controller,
}: {
  notice: NoticeSummary;
  controller: NoticeManagementController;
}) {
  const { t } = useTranslate('admin');
  return (
    <TableCell align="right" sx={{ whiteSpace: 'nowrap' }}>
      <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
        {controller.permissions.canOpenDetail ? (
          <Action
            title={t('notice.viewDetail')}
            icon="solar:eye-bold"
            onClick={() => controller.state.setDetailId(notice.notice_id)}
          />
        ) : null}
        {controller.permissions.canViewReaders ? (
          <Action
            title={t('notice.readers')}
            icon="solar:users-group-rounded-bold"
            onClick={() => controller.actions.openReaders(notice)}
          />
        ) : null}
        {controller.permissions.canEdit ? (
          <Action
            title={t('common.edit')}
            icon="solar:pen-bold"
            onClick={() => controller.state.setEditingId(notice.notice_id)}
          />
        ) : null}
        {controller.permissions.canRemove ? (
          <Action
            title={t('common.delete')}
            icon="solar:trash-bin-trash-bold"
            color="error"
            onClick={() => controller.state.setDeleteTarget(notice)}
          />
        ) : null}
      </Box>
    </TableCell>
  );
}

type ActionProps = { title: string; icon: IconifyName; color?: 'error'; onClick: () => void };
function Action({ title, icon, color, onClick }: ActionProps) {
  return (
    <Tooltip title={title}>
      <IconButton aria-label={title} color={color} onClick={onClick}>
        <Iconify icon={icon} />
      </IconButton>
    </Tooltip>
  );
}
