'use client';

import type { MouseEvent as ReactMouseEvent } from 'react';
import type { MoveDirectoryBrowser } from 'src/features/file-management';

import Box from '@mui/material/Box';
import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import Tooltip from '@mui/material/Tooltip';
import ButtonBase from '@mui/material/ButtonBase';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import Breadcrumbs from '@mui/material/Breadcrumbs';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import ListItemButton from '@mui/material/ListItemButton';

import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { FileThumbnail } from 'src/shared/ui/file-thumbnail';

import { ROOT_DIRECTORY_ID } from 'src/features/file-management';

import {
  MoveFolderContextMenu,
  MoveDirectoryContextMenu,
  useMoveDestinationContextMenus,
} from './move-destination-context-menu';

const MOVE_BROWSER_HEIGHT = 320;
const FOLDER_THUMBNAIL_SIZE = 32;
const EMPTY_FOLDER_THUMBNAIL_SIZE = 48;

type MoveDestinationBrowserProps = Readonly<{
  browser: MoveDirectoryBrowser;
  canMoveHere: boolean;
  canCreateFolder: boolean;
  disabled: boolean;
  onNavigate: (directoryId: string) => void;
  onCreateFolder: () => void;
  onMoveHere: () => void;
}>;

type MoveDirectoryNavigationProps = Pick<
  MoveDestinationBrowserProps,
  'browser' | 'disabled' | 'onNavigate'
>;

export function MoveDestinationBrowser({
  browser,
  canMoveHere,
  canCreateFolder,
  disabled,
  onNavigate,
  onCreateFolder,
  onMoveHere,
}: MoveDestinationBrowserProps) {
  const contextMenus = useMoveDestinationContextMenus({
    browser,
    canMoveHere,
    canCreateFolder,
    disabled,
  });
  return (
    <Box sx={{ border: 1, borderColor: 'divider', borderRadius: 1, overflow: 'hidden' }}>
      <DirectoryNavigation {...{ browser, disabled, onNavigate }} />
      <Scrollbar
        onContextMenu={contextMenus.onDirectoryContextMenu}
        sx={{ height: MOVE_BROWSER_HEIGHT }}
      >
        {browser.children.length ? (
          <FolderList
            folders={browser.children}
            disabled={disabled}
            onNavigate={onNavigate}
            onFolderContextMenu={contextMenus.onFolderContextMenu}
          />
        ) : (
          <EmptyDirectory />
        )}
      </Scrollbar>
      <DestinationSummary browser={browser} />
      <MoveDestinationContextMenus
        browser={browser}
        canMoveHere={canMoveHere}
        canCreateFolder={canCreateFolder}
        disabled={disabled}
        onNavigate={onNavigate}
        onCreateFolder={onCreateFolder}
        onMoveHere={onMoveHere}
        contextMenus={contextMenus}
      />
    </Box>
  );
}

type MoveDestinationContextMenusProps = Pick<
  MoveDestinationBrowserProps,
  | 'browser'
  | 'canMoveHere'
  | 'canCreateFolder'
  | 'disabled'
  | 'onNavigate'
  | 'onCreateFolder'
  | 'onMoveHere'
> &
  Readonly<{
    contextMenus: ReturnType<typeof useMoveDestinationContextMenus>;
  }>;

function MoveDestinationContextMenus({
  browser,
  canMoveHere,
  canCreateFolder,
  disabled,
  onNavigate,
  onCreateFolder,
  onMoveHere,
  contextMenus,
}: MoveDestinationContextMenusProps) {
  const { contextMenu } = contextMenus;
  if (!contextMenu) return null;
  return contextMenu.kind === 'directory' ? (
    <MoveDirectoryContextMenu
      browser={browser}
      canMoveHere={canMoveHere}
      canCreateFolder={canCreateFolder}
      disabled={disabled}
      position={contextMenu.position}
      onClose={contextMenus.close}
      onNavigate={onNavigate}
      onCreateFolder={onCreateFolder}
      onMoveHere={onMoveHere}
    />
  ) : (
    <MoveFolderContextMenu
      folder={contextMenu.folder}
      disabled={disabled}
      position={contextMenu.position}
      onClose={contextMenus.close}
      onNavigate={onNavigate}
    />
  );
}

function DirectoryNavigation({ browser, disabled, onNavigate }: MoveDirectoryNavigationProps) {
  const { t } = useTranslate('admin');
  const atRoot = browser.currentId === ROOT_DIRECTORY_ID;
  return (
    <Stack
      direction="row"
      spacing={1}
      alignItems="center"
      sx={{ p: 1, borderBottom: 1, borderColor: 'divider' }}
    >
      <Tooltip title={t('file.actions.upOneLevel')}>
        <span>
          <IconButton
            size="small"
            aria-label={t('file.actions.upOneLevel')}
            disabled={disabled || atRoot}
            onClick={() => onNavigate(browser.parentId)}
          >
            <Iconify icon="eva:arrow-ios-back-fill" />
          </IconButton>
        </span>
      </Tooltip>
      <DirectoryBreadcrumbs {...{ browser, disabled, onNavigate }} />
    </Stack>
  );
}

function DirectoryBreadcrumbs({ browser, disabled, onNavigate }: MoveDirectoryNavigationProps) {
  const { t } = useTranslate('admin');
  const items = [
    { id: ROOT_DIRECTORY_ID, name: t('file.root') },
    ...browser.trail.map((entry) => ({ id: entry.id, name: entry.name })),
  ];
  return (
    <Box sx={{ minWidth: 0, flex: 1, overflowX: 'auto' }}>
      <Breadcrumbs
        separator={<Iconify icon="eva:arrow-ios-forward-fill" width={16} />}
        aria-label={t('file.moveDestination')}
        sx={{ '& .MuiBreadcrumbs-ol': { flexWrap: 'nowrap' } }}
      >
        {items.map((item, index) => {
          const current = index === items.length - 1;
          return current ? (
            <Typography key={item.id} variant="body2" noWrap color="text.primary">
              {item.name}
            </Typography>
          ) : (
            <ButtonBase
              key={item.id}
              disabled={disabled}
              onClick={() => onNavigate(item.id)}
              sx={{ color: 'primary.main' }}
            >
              <Typography variant="body2" noWrap>
                {item.name}
              </Typography>
            </ButtonBase>
          );
        })}
      </Breadcrumbs>
    </Box>
  );
}

function FolderList({
  folders,
  disabled,
  onNavigate,
  onFolderContextMenu,
}: Readonly<{
  folders: MoveDirectoryBrowser['children'];
  disabled: boolean;
  onNavigate: (directoryId: string) => void;
  onFolderContextMenu: (
    event: ReactMouseEvent<HTMLElement>,
    folder: MoveDirectoryBrowser['children'][number]
  ) => void;
}>) {
  return (
    <List disablePadding>
      {folders.map((folder) => (
        <ListItemButton
          key={folder.id}
          divider
          disabled={disabled}
          onClick={() => onNavigate(folder.id)}
          onContextMenu={(event) => onFolderContextMenu(event, folder)}
          sx={{ minHeight: 56, px: 2 }}
        >
          <ListItemIcon sx={{ minWidth: 44 }}>
            <FileThumbnail
              file="folder"
              sx={{ width: FOLDER_THUMBNAIL_SIZE, height: FOLDER_THUMBNAIL_SIZE }}
            />
          </ListItemIcon>
          <ListItemText primary={folder.name} primaryTypographyProps={{ noWrap: true }} />
          <Iconify icon="eva:arrow-ios-forward-fill" width={18} />
        </ListItemButton>
      ))}
    </List>
  );
}

function EmptyDirectory() {
  const { t } = useTranslate('admin');
  return (
    <Stack
      alignItems="center"
      justifyContent="center"
      spacing={1}
      sx={{ minHeight: MOVE_BROWSER_HEIGHT, color: 'text.secondary' }}
    >
      <FileThumbnail
        file="folder"
        sx={{ width: EMPTY_FOLDER_THUMBNAIL_SIZE, height: EMPTY_FOLDER_THUMBNAIL_SIZE }}
      />
      <Typography variant="body2">{t('file.noChildFolders')}</Typography>
    </Stack>
  );
}

function DestinationSummary({ browser }: { browser: MoveDirectoryBrowser }) {
  const { t } = useTranslate('admin');
  const path = [t('file.root'), ...browser.trail.map((entry) => entry.name)].join(' / ');
  return (
    <Stack
      direction="row"
      spacing={1}
      alignItems="center"
      sx={{ p: 1.5, borderTop: 1, borderColor: 'divider', bgcolor: 'action.hover' }}
    >
      <Typography variant="caption" color="text.secondary" sx={{ flexShrink: 0 }}>
        {t('file.moveDestination')}
      </Typography>
      <Typography variant="body2" fontWeight={600} noWrap title={path} sx={{ minWidth: 0 }}>
        {path}
      </Typography>
    </Stack>
  );
}
