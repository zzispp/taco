import type { FileEntry } from 'src/entities/file';
import type { FileManagerController } from 'src/features/file-management';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { vi, it, expect, describe, beforeEach } from 'vitest';

type ContextMenuHandler = (event: unknown, entry?: FileEntry) => void;

const state = vi.hoisted(() => ({
  directoryContextMenu: vi.fn(),
  entryContextMenu: vi.fn(),
  scrollbarHandlers: [] as Array<ContextMenuHandler | undefined>,
  rowHandlers: [] as Array<Readonly<{ entry: FileEntry; handler: ContextMenuHandler }>>,
  cardHandlers: [] as Array<Readonly<{ entry: FileEntry; handler: ContextMenuHandler }>>,
}));

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: (key: string) => key }),
}));

vi.mock('src/shared/ui/scrollbar', () => ({
  Scrollbar: ({
    children,
    onContextMenu,
  }: {
    children: React.ReactNode;
    onContextMenu?: ContextMenuHandler;
  }) => {
    state.scrollbarHandlers.push(onContextMenu);
    return createElement('div', null, children);
  },
}));

vi.mock('src/shared/ui/table', () => ({
  TableNoData: () => null,
  TableHeadCustom: () => null,
  CursorPagination: () => null,
}));

vi.mock('src/features/file-management', () => ({
  canSelectFileEntries: () => false,
}));

vi.mock('./entry-item', () => ({
  FileEntryRow: ({
    entry,
    onContextMenu,
  }: {
    entry: FileEntry;
    onContextMenu: ContextMenuHandler;
  }) => {
    state.rowHandlers.push({ entry, handler: onContextMenu });
    return null;
  },
  FileEntryCard: ({
    entry,
    onContextMenu,
  }: {
    entry: FileEntry;
    onContextMenu: ContextMenuHandler;
  }) => {
    state.cardHandlers.push({ entry, handler: onContextMenu });
    return null;
  },
}));

vi.mock('./file-manager-context-menu', () => ({
  useFileManagerContextMenu: () => ({
    target: null,
    onDirectoryContextMenu: state.directoryContextMenu,
    onEntryContextMenu: state.entryContextMenu,
    close: vi.fn(),
  }),
  FileManagerContextMenus: () => null,
}));

import { FileEntryCollection } from './entry-collection';

const FILE_ENTRY = { id: 'file-1', type: 'file' } as FileEntry;

describe('file entry collection context menus', () => {
  beforeEach(() => {
    state.directoryContextMenu.mockReset();
    state.entryContextMenu.mockReset();
    state.scrollbarHandlers.length = 0;
    state.rowHandlers.length = 0;
    state.cardHandlers.length = 0;
  });

  it('routes the table scrollbar blank area to the shared directory menu', () => {
    renderCollection('list');
    const event = {};

    state.scrollbarHandlers[0]?.(event);

    expect(state.directoryContextMenu).toHaveBeenCalledOnce();
    expect(state.directoryContextMenu).toHaveBeenCalledWith(event);
  });

  it('routes table rows and grid cards to the shared entry menu', () => {
    renderCollection('list');
    const rowEvent = {};
    state.rowHandlers[0]?.handler(rowEvent, FILE_ENTRY);

    expect(state.entryContextMenu).toHaveBeenLastCalledWith(rowEvent, FILE_ENTRY);

    renderCollection('grid');
    const cardEvent = {};
    state.cardHandlers[0]?.handler(cardEvent, FILE_ENTRY);

    expect(state.entryContextMenu).toHaveBeenLastCalledWith(cardEvent, FILE_ENTRY);
  });
});

function renderCollection(viewMode: 'list' | 'grid') {
  renderToStaticMarkup(
    createElement(FileEntryCollection, { controller: collectionController(viewMode) })
  );
}

function collectionController(viewMode: 'list' | 'grid') {
  return {
    state: {
      mode: 'active',
      viewMode,
      table: {
        selected: [],
        order: 'asc',
        orderBy: 'name',
        onSelectAllRows: vi.fn(),
        onSort: vi.fn(),
        limit: 20,
        visitedBatchIndex: 0,
        onPreviousCursor: vi.fn(),
        onNextCursor: vi.fn(),
        onChangeLimit: vi.fn(),
      },
    },
    permissions: { canList: true },
    resources: {
      entries: {
        items: [FILE_ENTRY],
        isLoading: false,
        isValidating: false,
        itemCount: 1,
        hasPrevious: false,
        hasNext: false,
      },
    },
  } as unknown as FileManagerController;
}
