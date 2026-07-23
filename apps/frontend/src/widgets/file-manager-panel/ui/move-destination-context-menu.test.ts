import type { MoveDirectoryBrowser } from 'src/features/file-management';

import { createElement } from 'react';
import { vi, it, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

vi.mock('@mui/material/Menu', async () => {
  const { createElement: create } = await import('react');
  return {
    default: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
      open ? create('div', null, children) : null,
  };
});

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));

vi.mock('src/shared/ui/iconify', async () => {
  const { createElement: create } = await import('react');
  return {
    Iconify: ({ icon }: { icon: string }) => create('i', { 'data-icon': icon }),
  };
});

import { ROOT_DIRECTORY_ID } from 'src/features/file-management';

import {
  MoveFolderContextMenu,
  MoveDirectoryContextMenu,
  selectMoveFolderContextMenu,
  selectMoveDirectoryContextMenu,
} from './move-destination-context-menu';

const ROOT_BROWSER: MoveDirectoryBrowser = {
  currentId: ROOT_DIRECTORY_ID,
  parentId: ROOT_DIRECTORY_ID,
  trail: [],
  children: [
    {
      id: 'folder-a',
      space_id: 'space-a',
      parent_id: ROOT_DIRECTORY_ID,
      name: '项目资料',
      type: 'folder',
    },
  ],
};

const NESTED_BROWSER: MoveDirectoryBrowser = {
  ...ROOT_BROWSER,
  currentId: 'folder-a',
  parentId: ROOT_DIRECTORY_ID,
  trail: [ROOT_BROWSER.children[0]],
};

const POSITION = { mouseX: 128, mouseY: 256 };

describe('move destination context menus', () => {
  it('creates a folder menu state that replaces a directory menu state', () => {
    const directoryMenu = selectMoveDirectoryContextMenu({
      browser: ROOT_BROWSER,
      canMoveHere: true,
      canCreateFolder: false,
      disabled: false,
      position: POSITION,
    });
    const folderMenu = selectMoveFolderContextMenu({
      folder: ROOT_BROWSER.children[0],
      disabled: false,
      position: POSITION,
    });

    expect(directoryMenu).toEqual({ kind: 'directory', position: POSITION });
    expect(folderMenu).toEqual({
      kind: 'folder',
      folder: ROOT_BROWSER.children[0],
      position: POSITION,
    });
    expect(folderMenu).not.toEqual(directoryMenu);
  });

  it('does not create a root directory menu when no command is available', () => {
    expect(
      selectMoveDirectoryContextMenu({
        browser: ROOT_BROWSER,
        canMoveHere: false,
        canCreateFolder: false,
        disabled: false,
        position: POSITION,
      })
    ).toBeNull();
  });

  it('creates a root directory menu when folder creation is available', () => {
    expect(
      selectMoveDirectoryContextMenu({
        browser: ROOT_BROWSER,
        canMoveHere: false,
        canCreateFolder: true,
        disabled: false,
        position: POSITION,
      })
    ).toEqual({ kind: 'directory', position: POSITION });
  });

  it('renders the new-folder action only when creation is permitted', () => {
    const withCreation = renderDirectoryMenu({
      browser: ROOT_BROWSER,
      canMoveHere: false,
      canCreateFolder: true,
    });
    const withoutCreation = renderDirectoryMenu({
      browser: ROOT_BROWSER,
      canMoveHere: true,
      canCreateFolder: false,
    });

    expect(withCreation).toContain('file.actions.newFolder');
    expect(withCreation).toContain('solar:add-folder-bold');
    expect(withoutCreation).not.toContain('file.actions.newFolder');
    expect(withoutCreation).toContain('file.actions.moveHere');
  });

  it('keeps the parent command available in an invalid nested destination', () => {
    expect(
      selectMoveDirectoryContextMenu({
        browser: NESTED_BROWSER,
        canMoveHere: false,
        canCreateFolder: false,
        disabled: false,
        position: POSITION,
      })
    ).toEqual({ kind: 'directory', position: POSITION });
  });

  it('keeps parent navigation and moving commands in nested directories', () => {
    const html = renderDirectoryMenu({
      browser: NESTED_BROWSER,
      canMoveHere: false,
      canCreateFolder: false,
    });

    expect(html).toContain('file.actions.upOneLevel');
    expect(html).toContain('file.actions.moveHere');
  });

  it('keeps a folder-row menu limited to opening that folder', () => {
    const html = renderToStaticMarkup(
      createElement(MoveFolderContextMenu, {
        folder: ROOT_BROWSER.children[0],
        disabled: false,
        position: POSITION,
        onClose: vi.fn(),
        onNavigate: vi.fn(),
      })
    );

    expect(html).toContain('file.actions.open');
    expect(html).not.toContain('file.actions.newFolder');
    expect(html).not.toContain('file.actions.moveHere');
  });

  it('does not create a disabled folder command menu', () => {
    expect(
      selectMoveFolderContextMenu({
        folder: ROOT_BROWSER.children[0],
        disabled: true,
        position: POSITION,
      })
    ).toBeNull();
  });
});

function renderDirectoryMenu({
  browser,
  canMoveHere,
  canCreateFolder,
}: Readonly<{
  browser: MoveDirectoryBrowser;
  canMoveHere: boolean;
  canCreateFolder: boolean;
}>) {
  return renderToStaticMarkup(
    createElement(MoveDirectoryContextMenu, {
      browser,
      canMoveHere,
      canCreateFolder,
      disabled: false,
      position: POSITION,
      onClose: vi.fn(),
      onNavigate: vi.fn(),
      onCreateFolder: vi.fn(),
      onMoveHere: vi.fn(),
    })
  );
}
