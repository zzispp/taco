import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { vi, it, expect, describe, beforeEach } from 'vitest';

const state = vi.hoisted(() => ({ openedMenuCount: 0 }));

vi.mock('@mui/material/Menu', () => ({
  default: ({ open }: { open: boolean }) => {
    if (open) state.openedMenuCount += 1;
    return null;
  },
}));

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));

import { entryContextMenuPosition, preventBrowserContextMenu } from './entry-context-menu';
import {
  entryContextMenuTarget,
  FileManagerContextMenus,
  directoryContextMenuTarget,
} from './file-manager-context-menu';

const POSITION = { mouseX: 320, mouseY: 176 };
const FILE_ENTRY = { id: 'file-1', type: 'file', preview_supported: false } as FileEntry;

beforeEach(() => {
  state.openedMenuCount = 0;
});

describe('file entry context menu', () => {
  it('anchors the MUI menu at the browser context-menu coordinates', () => {
    expect(entryContextMenuPosition({ clientX: 320, clientY: 176 })).toEqual({
      mouseX: 320,
      mouseY: 176,
    });
  });

  it('keeps an entry menu event from opening its containing directory menu', () => {
    const event = { preventDefault: vi.fn(), stopPropagation: vi.fn() };

    preventBrowserContextMenu(event);

    expect(event.preventDefault).toHaveBeenCalledOnce();
    expect(event.stopPropagation).toHaveBeenCalledOnce();
  });

  it('does not create blank directory or entry menus when no action is available', () => {
    const controller = contextMenuController();

    expect(directoryContextMenuTarget(controller, POSITION)).toBeNull();
    expect(entryContextMenuTarget(FILE_ENTRY, controller, POSITION)).toBeNull();
  });

  it('renders only the selected menu target', () => {
    const controller = contextMenuController({ canQuery: true });
    const target = entryContextMenuTarget(FILE_ENTRY, controller, POSITION);

    renderToStaticMarkup(
      createElement(FileManagerContextMenus, { controller, target, onClose: vi.fn() })
    );

    expect(state.openedMenuCount).toBe(1);
  });
});

function contextMenuController(permissions: Partial<FileManagerController['permissions']> = {}) {
  return {
    state: { mode: 'trash', parentId: null },
    permissions: {
      canList: false,
      canQuery: false,
      canDownload: false,
      canUpload: false,
      canAddFolder: false,
      canEdit: false,
      canRemove: false,
      canRestore: false,
      canPurge: false,
      ...permissions,
    },
    actions: { openDetail: vi.fn() },
  } as unknown as FileManagerController;
}
