'use client';

import type { MouseEvent as ReactMouseEvent } from 'react';
import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import { useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Menu from '@mui/material/Menu';
import Divider from '@mui/material/Divider';
import MenuItem from '@mui/material/MenuItem';
import ListItemText from '@mui/material/ListItemText';
import ListItemIcon from '@mui/material/ListItemIcon';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  entryContextMenuPosition,
  preventBrowserContextMenu,
  type EntryContextMenuPosition,
} from './entry-context-menu';
import {
  entryMenuContent,
  hasEntryMenuActions,
  directoryMenuContent,
  hasDirectoryMenuActions,
  type FileManagerContextMenuItem,
  type FileManagerContextMenuContent,
} from './file-manager-context-menu-actions';

type FileManagerContextMenuTarget =
  | Readonly<{ kind: 'directory'; position: EntryContextMenuPosition }>
  | Readonly<{ kind: 'entry'; entry: FileEntry; position: EntryContextMenuPosition }>;

export function directoryContextMenuTarget(
  controller: FileManagerController,
  position: EntryContextMenuPosition
): FileManagerContextMenuTarget | null {
  return hasDirectoryMenuActions(controller) ? { kind: 'directory', position } : null;
}

export function entryContextMenuTarget(
  entry: FileEntry,
  controller: FileManagerController,
  position: EntryContextMenuPosition
): FileManagerContextMenuTarget | null {
  return hasEntryMenuActions(entry, controller) ? { kind: 'entry', entry, position } : null;
}

export function useFileManagerContextMenu(controller: FileManagerController) {
  const [target, setTarget] = useState<FileManagerContextMenuTarget | null>(null);
  const onDirectoryContextMenu = useCallback(
    (event: ReactMouseEvent<HTMLElement>) => {
      openContextMenu(
        directoryContextMenuTarget(controller, entryContextMenuPosition(event)),
        event,
        setTarget
      );
    },
    [controller]
  );
  const onEntryContextMenu = useCallback(
    (event: ReactMouseEvent<HTMLElement>, entry: FileEntry) => {
      openContextMenu(
        entryContextMenuTarget(entry, controller, entryContextMenuPosition(event)),
        event,
        setTarget
      );
    },
    [controller]
  );
  const close = useCallback(() => setTarget(null), []);

  return { target, onDirectoryContextMenu, onEntryContextMenu, close };
}

export function FileManagerContextMenus({
  controller,
  target,
  onClose,
}: Readonly<{
  controller: FileManagerController;
  target: FileManagerContextMenuTarget | null;
  onClose: () => void;
}>) {
  const { t } = useTranslate('admin');
  if (!target) return null;
  const content =
    target.kind === 'entry'
      ? entryMenuContent(target.entry, controller, t)
      : directoryMenuContent(controller, t);

  return <ContextMenu position={target.position} onClose={onClose} content={content} />;
}

function openContextMenu(
  target: FileManagerContextMenuTarget | null,
  event: ReactMouseEvent<HTMLElement>,
  setTarget: (target: FileManagerContextMenuTarget | null) => void
) {
  if (!target) {
    setTarget(null);
    return;
  }
  preventBrowserContextMenu(event);
  setTarget(target);
}

function ContextMenu({
  position,
  onClose,
  content,
}: Readonly<{
  position: EntryContextMenuPosition;
  onClose: () => void;
  content: FileManagerContextMenuContent[];
}>) {
  return (
    <Menu
      open={content.length > 0}
      onClose={onClose}
      anchorReference="anchorPosition"
      anchorPosition={{ top: position.mouseY, left: position.mouseX }}
      slotProps={{ backdrop: { invisible: true }, paper: { sx: { minWidth: 190 } } }}
    >
      {content.map((item, index) =>
        item === 'divider' ? (
          <Divider key={`divider-${index}`} component="li" />
        ) : (
          <ContextAction
            key={item.id}
            {...item}
            onClick={() => invokeContextAction(item, onClose)}
          />
        )
      )}
    </Menu>
  );
}

function invokeContextAction(item: FileManagerContextMenuItem, onClose: () => void) {
  onClose();
  item.onClick();
}

function ContextAction({
  icon,
  label,
  onClick,
  destructive = false,
}: Readonly<FileManagerContextMenuItem>) {
  return (
    <MenuItem onClick={onClick} sx={destructive ? { color: 'error.main' } : undefined}>
      <ListItemIcon>
        <Box component="span" sx={{ display: 'inline-flex' }}>
          <Iconify icon={icon} width={18} />
        </Box>
      </ListItemIcon>
      <ListItemText>{label}</ListItemText>
    </MenuItem>
  );
}
