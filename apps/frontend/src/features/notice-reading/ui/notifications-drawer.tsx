'use client';

import type { IconButtonProps } from '@mui/material/IconButton';
import type { NoticeTab } from '../model/notice-tabs';

import { useState } from 'react';
import { m } from 'framer-motion';
import { useBoolean } from 'minimal-shared/hooks';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Badge from '@mui/material/Badge';
import Alert from '@mui/material/Alert';
import Drawer from '@mui/material/Drawer';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import CircularProgress from '@mui/material/CircularProgress';

import { Label } from 'src/shared/ui/label';
import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { paths } from 'src/shared/routes/paths';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useRouter } from 'src/shared/routes/hooks';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { varTap, varHover, transitionTap } from 'src/shared/ui/animate';

import { useHasPermission } from 'src/entities/session';
import {
  useNotice,
  useNoticeTop,
  NOTICE_PERMISSION,
  NoticeDetailDrawer,
} from 'src/entities/notice';

import { NotificationItem } from './notification-item';
import { markNoticeRead, markAllNoticesRead } from '../api';
import { filterNoticeTopItems } from '../model/notice-tabs';

export function NotificationsDrawer({ sx, ...other }: IconButtonProps) {
  const controller = useNotificationsDrawerController();
  const { t } = useTranslate('admin');
  return (
    <>
      <IconButton
        component={m.button}
        whileTap={varTap(0.96)}
        whileHover={varHover(1.04)}
        transition={transitionTap()}
        aria-label={t('notice.notifications')}
        onClick={controller.open.onTrue}
        sx={sx}
        {...other}
      >
        <Badge badgeContent={controller.unreadCount} color="error">
          <Iconify width={24} icon="solar:bell-bing-bold-duotone" />
        </Badge>
      </IconButton>
      <NotificationsDrawerPanel controller={controller} />
      <NoticeDetailDrawer
        open={Boolean(controller.selectedId)}
        notice={controller.detail.data}
        loading={controller.detail.isLoading}
        error={controller.detail.error}
        onClose={() => controller.setSelectedId(null)}
      />
    </>
  );
}

function useNotificationsDrawerController() {
  const router = useRouter();
  const open = useBoolean();
  const [tab, setTab] = useState<NoticeTab>('all');
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [pending, setPending] = useState<string | null>(null);
  const top = useNoticeTop();
  const detail = useNotice(selectedId, true);
  const canList = useHasPermission(NOTICE_PERMISSION.LIST);
  const items = top.data?.items ?? [];
  const visibleItems = filterNoticeTopItems(items, tab);
  const unreadCount = top.data?.unread_count ?? 0;
  const viewAll = () => {
    open.onFalse();
    router.push(paths.dashboard.admin.notices);
  };
  return {
    open,
    tab,
    setTab,
    selectedId,
    setSelectedId,
    pending,
    setPending,
    top,
    detail,
    canList,
    items,
    visibleItems,
    unreadCount,
    viewAll,
  };
}

type DrawerController = ReturnType<typeof useNotificationsDrawerController>;

function NotificationsDrawerPanel({ controller }: { controller: DrawerController }) {
  return (
    <Drawer
      open={controller.open.value}
      onClose={controller.open.onFalse}
      anchor="right"
      slotProps={{
        backdrop: { invisible: true },
        paper: { sx: { width: 1, maxWidth: 420 } },
      }}
    >
      <DrawerHeader controller={controller} />
      <DrawerTabs controller={controller} />
      <DrawerList controller={controller} />
      {controller.canList ? <DrawerFooter controller={controller} /> : null}
    </Drawer>
  );
}

function DrawerHeader({ controller }: { controller: DrawerController }) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ py: 2, pr: 1, pl: 2.5, minHeight: 68, display: 'flex', alignItems: 'center' }}>
      <Typography variant="h6" sx={{ flexGrow: 1 }}>
        {t('notice.notifications')}
      </Typography>
      {controller.unreadCount > 0 ? (
        <Tooltip title={t('notice.markAllRead')}>
          <span>
            <IconButton
              color="primary"
              disabled={controller.pending === 'read-all'}
              onClick={() => handleMarkAllRead(controller)}
            >
              <Iconify icon="eva:done-all-fill" />
            </IconButton>
          </span>
        </Tooltip>
      ) : null}
      <IconButton
        aria-label={t('common.close')}
        onClick={controller.open.onFalse}
        sx={{ display: { xs: 'inline-flex', sm: 'none' } }}
      >
        <Iconify icon="mingcute:close-line" />
      </IconButton>
    </Box>
  );
}

function DrawerTabs({ controller }: { controller: DrawerController }) {
  const { t } = useTranslate('admin');
  const tabs = [
    { value: 'all' as const, label: t('common.all'), count: controller.items.length },
    { value: 'unread' as const, label: t('notice.unread'), count: controller.unreadCount },
  ];
  return (
    <Tabs
      variant="fullWidth"
      value={controller.tab}
      onChange={(_, value: NoticeTab) => controller.setTab(value)}
    >
      {tabs.map((tab) => (
        <Tab
          key={tab.value}
          value={tab.value}
          label={tab.label}
          iconPosition="end"
          icon={
            <Label
              variant={tab.value === controller.tab ? 'filled' : 'soft'}
              color={tab.value === 'unread' ? 'info' : 'default'}
            >
              {tab.count}
            </Label>
          }
        />
      ))}
    </Tabs>
  );
}

function DrawerList({ controller }: { controller: DrawerController }) {
  const { t } = useTranslate('admin');
  if (controller.top.error) {
    return (
      <Alert severity="error" sx={{ m: 2 }}>
        {getErrorMessage(controller.top.error)}
      </Alert>
    );
  }
  if (controller.top.isLoading) {
    return (
      <Box sx={{ py: 10, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    );
  }
  return (
    <Scrollbar>
      <Box component="ul" sx={{ m: 0, p: 0, listStyle: 'none' }}>
        {controller.visibleItems.map((notice) => (
          <Box component="li" key={notice.notice_id}>
            <NotificationItem
              notice={notice}
              disabled={controller.pending === `read:${notice.notice_id}`}
              onClick={() => handleOpenNotice(controller, notice.notice_id, notice.is_read)}
            />
          </Box>
        ))}
      </Box>
      {!controller.top.isLoading && controller.visibleItems.length === 0 ? (
        <Typography sx={{ py: 8, textAlign: 'center', color: 'text.secondary' }}>
          {t('notice.noNotifications')}
        </Typography>
      ) : null}
    </Scrollbar>
  );
}

function DrawerFooter({ controller }: { controller: DrawerController }) {
  const { t } = useTranslate('admin');
  return (
    <Box sx={{ p: 1 }}>
      <Button fullWidth size="large" onClick={controller.viewAll}>
        {t('notice.viewAll')}
      </Button>
    </Box>
  );
}

async function handleOpenNotice(controller: DrawerController, id: string, isRead: boolean) {
  if (isRead) {
    controller.open.onFalse();
    controller.setSelectedId(id);
    return;
  }
  controller.setPending(`read:${id}`);
  try {
    await markNoticeRead(id);
    controller.open.onFalse();
    controller.setSelectedId(id);
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    controller.setPending(null);
  }
}

async function handleMarkAllRead(controller: DrawerController) {
  controller.setPending('read-all');
  try {
    await markAllNoticesRead();
  } catch (error) {
    toast.error(getErrorMessage(error));
  } finally {
    controller.setPending(null);
  }
}
