import type { NoticeTopItem } from 'src/entities/notice';

import Box from '@mui/material/Box';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';
import { fToNow } from 'src/shared/lib/format-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { NOTICE_TYPE, noticeTypeColors, noticeTypeTranslationKeys } from 'src/entities/notice';

type NotificationItemProps = Readonly<{
  notice: NoticeTopItem;
  disabled: boolean;
  onClick: () => void;
}>;

export function NotificationItem({ notice, disabled, onClick }: NotificationItemProps) {
  return (
    <ListItemButton
      disableRipple
      disabled={disabled}
      onClick={onClick}
      sx={(theme) => ({
        p: 2.5,
        gap: 2,
        alignItems: 'flex-start',
        borderBottom: `dashed 1px ${theme.vars.palette.divider}`,
      })}
    >
      <NotificationIcon notice={notice} />
      <NotificationText notice={notice} />
      <UnreadIndicator visible={!notice.is_read} />
    </ListItemButton>
  );
}

function NotificationIcon({ notice }: { notice: NoticeTopItem }) {
  const icon =
    notice.notice_type === NOTICE_TYPE.ANNOUNCEMENT
      ? 'solar:volume-loud-bold'
      : 'solar:bell-bing-bold';
  return (
    <Box
      sx={{
        width: 40,
        height: 40,
        flexShrink: 0,
        display: 'flex',
        borderRadius: '50%',
        alignItems: 'center',
        justifyContent: 'center',
        color: `${noticeTypeColors[notice.notice_type]}.main`,
        bgcolor: 'background.neutral',
      }}
    >
      <Iconify icon={icon} />
    </Box>
  );
}

function NotificationText({ notice }: { notice: NoticeTopItem }) {
  const { t } = useTranslate('admin');
  const secondary = (
    <Box component="span" sx={{ mt: 0.75, gap: 0.75, display: 'flex', alignItems: 'center' }}>
      <Label color={noticeTypeColors[notice.notice_type]} variant="soft">
        {t(noticeTypeTranslationKeys[notice.notice_type])}
      </Label>
      <Box component="span" sx={{ typography: 'caption', color: 'text.disabled' }}>
        {fToNow(notice.create_time)}
      </Box>
    </Box>
  );
  return (
    <ListItemText
      primary={notice.notice_title}
      secondary={secondary}
      slotProps={{
        primary: { sx: { pr: 2, typography: 'subtitle2' } },
        secondary: { component: 'div' },
      }}
    />
  );
}

function UnreadIndicator({ visible }: { visible: boolean }) {
  const { t } = useTranslate('admin');
  if (!visible) return null;
  return (
    <Box
      aria-label={t('notice.unread')}
      sx={{ mt: 1, width: 8, height: 8, flexShrink: 0, borderRadius: '50%', bgcolor: 'info.main' }}
    />
  );
}
