import type { FileManagerController } from 'src/features/file-management';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { it, vi, expect, describe, beforeEach } from 'vitest';

const state = vi.hoisted(() => ({ cancelTexts: [] as string[] }));

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({
    t: (key: string) => (key === 'file.actions.cancel' ? '取消' : key),
  }),
}));

vi.mock('src/shared/ui/custom-dialog', async () => {
  const { createElement: create } = await import('react');
  return {
    ConfirmDialog: ({ cancelText }: { cancelText?: string }) => {
      state.cancelTexts.push(cancelText ?? 'missing');
      return create('output', { 'data-cancel-text': cancelText ?? 'missing' });
    },
  };
});

vi.mock('./move-dialog', () => ({ MoveDialog: () => null }));
vi.mock('./upload-queue', () => ({ UploadQueue: () => null }));

import { FileManagerDialogs } from './dialogs';

describe('file manager dialogs', () => {
  beforeEach(() => {
    state.cancelTexts.length = 0;
  });

  it('uses the localized cancel text in the move-to-trash confirmation', () => {
    renderToStaticMarkup(createElement(FileManagerDialogs, { controller: createController() }));

    expect(state.cancelTexts).toEqual(['取消']);
  });
});

function createController() {
  return {
    state: {
      uploadOpen: false,
      uploadItems: [],
      folderOpen: false,
      folderName: '',
      mode: 'active',
      deleteTarget: { id: 'asset-1', name: 'budget.xlsx' },
      batchAction: null,
      table: { selected: [] },
    },
    resources: { spaceId: 'space-1' },
    actions: {
      closeUpload: vi.fn(),
      cancelUpload: vi.fn(),
      submitUpload: vi.fn(),
      closeFolderDialog: vi.fn(),
      submitFolder: vi.fn(),
      closeDelete: vi.fn(),
      deleteEntry: vi.fn(),
      closeBatchAction: vi.fn(),
      executeBatchAction: vi.fn(),
    },
    pending: new Set<string>(),
  } as unknown as FileManagerController;
}
