'use client';

import type { MouseEvent as ReactMouseEvent } from 'react';
import type { EntryContextMenuPosition } from './entry-context-menu';
import type { MoveDirectoryBrowser } from 'src/features/file-management';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Menu from '@mui/material/Menu';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import ListItemText from '@mui/material/ListItemText';
import ListItemIcon from '@mui/material/ListItemIcon';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { ROOT_DIRECTORY_ID } from 'src/features/file-management';

import { entryContextMenuPosition, preventBrowserContextMenu } from './entry-context-menu';

type MoveFolder = MoveDirectoryBrowser['children'][number];

export type MoveDestinationContextMenu =
  | Readonly<{
      kind: 'directory';
      position: EntryContextMenuPosition;
    }>
  | Readonly<{
      kind: 'folder';
      folder: MoveFolder;
      position: EntryContextMenuPosition;
    }>;

type SelectMoveDirectoryContextMenuOptions = Readonly<{
  browser: MoveDirectoryBrowser;
  canMoveHere: boolean;
  canCreateFolder: boolean;
  disabled: boolean;
  position: EntryContextMenuPosition;
}>;

type SelectMoveFolderContextMenuOptions = Readonly<{
  folder: MoveFolder;
  disabled: boolean;
  position: EntryContextMenuPosition;
}>;

type MoveDestinationContextMenuOptions = Readonly<{
  browser: MoveDirectoryBrowser;
  canMoveHere: boolean;
  canCreateFolder: boolean;
  disabled: boolean;
}>;

export function selectMoveDirectoryContextMenu({
  browser,
  canMoveHere,
  canCreateFolder,
  disabled,
  position,
}: SelectMoveDirectoryContextMenuOptions): MoveDestinationContextMenu | null {
  if (!canShowMoveDirectoryContextMenu(browser, canMoveHere, canCreateFolder, disabled))
    return null;
  return { kind: 'directory', position };
}

export function selectMoveFolderContextMenu({
  folder,
  disabled,
  position,
}: SelectMoveFolderContextMenuOptions): MoveDestinationContextMenu | null {
  if (disabled) return null;
  return { kind: 'folder', folder, position };
}

export function useMoveDestinationContextMenus({
  browser,
  canMoveHere,
  canCreateFolder,
  disabled,
}: MoveDestinationContextMenuOptions) {
  const [contextMenu, setContextMenu] = useState<MoveDestinationContextMenu | null>(null);
  const close = useCallback(() => setContextMenu(null), []);
  const onDirectoryContextMenu = useCallback(
    (event: ReactMouseEvent<HTMLElement>) => {
      preventBrowserContextMenu(event);
      setContextMenu(
        selectMoveDirectoryContextMenu({
          browser,
          canMoveHere,
          canCreateFolder,
          disabled,
          position: entryContextMenuPosition(event),
        })
      );
    },
    [browser, canMoveHere, canCreateFolder, disabled]
  );
  const onFolderContextMenu = useCallback(
    (event: ReactMouseEvent<HTMLElement>, folder: MoveFolder) => {
      preventBrowserContextMenu(event);
      setContextMenu(
        selectMoveFolderContextMenu({
          folder,
          disabled,
          position: entryContextMenuPosition(event),
        })
      );
    },
    [disabled]
  );
  return { contextMenu, close, onDirectoryContextMenu, onFolderContextMenu };
}

type MoveDirectoryContextMenuProps = Readonly<{
  browser: MoveDirectoryBrowser;
  canMoveHere: boolean;
  canCreateFolder: boolean;
  disabled: boolean;
  position: EntryContextMenuPosition | null;
  onClose: () => void;
  onNavigate: (directoryId: string) => void;
  onCreateFolder: () => void;
  onMoveHere: () => void;
}>;

export function MoveDirectoryContextMenu({
  browser,
  canMoveHere,
  canCreateFolder,
  disabled,
  position,
  onClose,
  onNavigate,
  onCreateFolder,
  onMoveHere,
}: MoveDirectoryContextMenuProps) {
  const { t } = useTranslate('admin');
  if (
    !position ||
    !canShowMoveDirectoryContextMenu(browser, canMoveHere, canCreateFolder, disabled)
  ) {
    return null;
  }
  const invoke = (action: () => void) => {
    onClose();
    action();
  };
  const atRoot = browser.currentId === ROOT_DIRECTORY_ID;
  return (
    <Menu {...menuPositionProps(position, onClose)}>
      {!atRoot ? (
        <MoveContextAction
          icon="eva:arrow-ios-back-fill"
          label={t('file.actions.upOneLevel')}
          disabled={disabled}
          onClick={() => invoke(() => onNavigate(browser.parentId))}
        />
      ) : null}
      {!atRoot && canCreateFolder ? <Divider component="li" /> : null}
      {canCreateFolder ? (
        <MoveContextAction
          icon="solar:add-folder-bold"
          label={t('file.actions.newFolder')}
          disabled={disabled}
          onClick={() => invoke(onCreateFolder)}
        />
      ) : null}
      {!atRoot || canCreateFolder ? <Divider component="li" /> : null}
      <MoveContextAction
        icon="eva:arrow-forward-fill"
        label={t('file.actions.moveHere')}
        disabled={disabled || !canMoveHere}
        onClick={() => invoke(onMoveHere)}
      />
    </Menu>
  );
}

type MoveFolderContextMenuProps = Readonly<{
  folder: MoveDirectoryBrowser['children'][number];
  disabled: boolean;
  position: EntryContextMenuPosition | null;
  onClose: () => void;
  onNavigate: (directoryId: string) => void;
}>;

export function MoveFolderContextMenu({
  folder,
  disabled,
  position,
  onClose,
  onNavigate,
}: MoveFolderContextMenuProps) {
  const { t } = useTranslate('admin');
  if (!position || disabled) return null;
  return (
    <Menu {...menuPositionProps(position, onClose)}>
      <MoveContextAction
        icon="eva:arrow-ios-forward-fill"
        label={t('file.actions.open')}
        disabled={disabled}
        onClick={() => {
          onClose();
          onNavigate(folder.id);
        }}
      />
    </Menu>
  );
}

function canShowMoveDirectoryContextMenu(
  browser: MoveDirectoryBrowser,
  canMoveHere: boolean,
  canCreateFolder: boolean,
  disabled: boolean
) {
  if (disabled) return false;
  return browser.currentId !== ROOT_DIRECTORY_ID || canMoveHere || canCreateFolder;
}

function menuPositionProps(position: EntryContextMenuPosition | null, onClose: () => void) {
  return {
    open: position !== null,
    onClose,
    anchorReference: 'anchorPosition' as const,
    anchorPosition: position ? { top: position.mouseY, left: position.mouseX } : undefined,
    slotProps: {
      backdrop: { invisible: true },
      paper: { sx: { minWidth: 190 } },
    },
  };
}

function MoveContextAction({
  icon,
  label,
  disabled,
  onClick,
}: Readonly<{
  icon: Parameters<typeof Iconify>[0]['icon'];
  label: string;
  disabled: boolean;
  onClick: () => void;
}>) {
  return (
    <MenuItem disabled={disabled} onClick={onClick}>
      <ListItemIcon>
        <Box component="span" sx={{ display: 'inline-flex' }}>
          <Iconify icon={icon} width={18} />
        </Box>
      </ListItemIcon>
      <ListItemText>{label}</ListItemText>
    </MenuItem>
  );
}
