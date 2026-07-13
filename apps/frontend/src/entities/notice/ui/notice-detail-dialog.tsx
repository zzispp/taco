'use client';

import type { Notice } from '../model/types';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Drawer from '@mui/material/Drawer';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import CircularProgress from '@mui/material/CircularProgress';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { Markdown } from 'src/shared/ui/markdown';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import {
  noticeTypeColors,
  noticeStatusColors,
  noticeTypeTranslationKeys,
  noticeStatusTranslationKeys,
} from '../model/constants';

type NoticeDetailDrawerProps = Readonly<{
  open: boolean;
  notice?: Notice;
  loading: boolean;
  error?: unknown;
  onClose: () => void;
}>;

export function NoticeDetailDrawer(props: NoticeDetailDrawerProps) {
  const { t } = useTranslate('admin');
  return (
    <Drawer
      anchor="right"
      open={props.open}
      onClose={props.onClose}
      slotProps={{
        backdrop: { invisible: true },
        paper: { sx: { width: { xs: 1, sm: '60vw', lg: '50vw' }, maxWidth: 760 } },
      }}
    >
      <Box sx={{ px: 2.5, minHeight: 68, display: 'flex', alignItems: 'center' }}>
        <Typography variant="h6" sx={{ flex: 1 }}>
          {t('notice.detailTitle')}
        </Typography>
        <IconButton aria-label={t('common.close')} onClick={props.onClose}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Box>
      <Divider />
      <Scrollbar>
        <Box sx={{ p: { xs: 2.5, sm: 4 }, minHeight: 320 }}>
          <NoticeDetailContent {...props} />
        </Box>
      </Scrollbar>
    </Drawer>
  );
}

function NoticeDetailContent(props: NoticeDetailDrawerProps) {
  const { t } = useTranslate('admin');
  if (props.loading) {
    return (
      <Box sx={{ py: 10, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    );
  }
  if (props.error) return <Alert severity="error">{getErrorMessage(props.error)}</Alert>;
  if (!props.notice) return null;
  return (
    <Box>
      <Typography variant="h5">{props.notice.notice_title}</Typography>
      <Box sx={{ mt: 1, gap: 1, display: 'flex', alignItems: 'center', flexWrap: 'wrap' }}>
        <Label color={noticeTypeColors[props.notice.notice_type]} variant="soft">
          {t(noticeTypeTranslationKeys[props.notice.notice_type])}
        </Label>
        <Label color={noticeStatusColors[props.notice.status]} variant="soft">
          {t(noticeStatusTranslationKeys[props.notice.status])}
        </Label>
        <Typography variant="caption" color="text.secondary">
          {props.notice.create_by} · {fAdminDateTime(props.notice.create_time)}
        </Typography>
      </Box>
      <Divider sx={{ my: 3 }} />
      {props.notice.notice_content ? (
        <Markdown allowRawHtml={false} sourceFormat="markdown">
          {props.notice.notice_content}
        </Markdown>
      ) : (
        <Typography color="text.secondary">{t('notice.emptyContent')}</Typography>
      )}
      {props.notice.remark ? (
        <Alert severity="info" sx={{ mt: 3 }}>
          {props.notice.remark}
        </Alert>
      ) : null}
    </Box>
  );
}
