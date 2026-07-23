import type { ReactNode } from 'react';
import type { FileEntry, FileOverview } from 'src/entities/file';

import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { it, vi, expect, describe, beforeEach } from 'vitest';

const state = vi.hoisted(() => ({
  buildEntryPath: vi.fn(
    (asset: Pick<FileEntry, 'id'>) => `/dashboard/file-manager?entry=${asset.id}`
  ),
  translate: vi.fn((key: string) => (key === 'file.recentAssets' ? '最近更新资产' : key)),
}));

vi.mock('src/features/file-management', () => ({
  buildFileManagerEntryPath: state.buildEntryPath,
}));

vi.mock('src/shared/i18n/use-locales', () => ({
  useTranslate: () => ({ t: state.translate }),
}));

vi.mock('src/entities/file', async () => {
  const { createElement: create } = await import('react');
  return {
    ManagedFileThumbnail: ({ entry: asset }: { entry: Pick<FileEntry, 'id'> }) =>
      create('span', { 'data-entry-id': asset.id }),
  };
});

vi.mock('src/shared/routes/components', async () => {
  const { createElement: create } = await import('react');
  return {
    RouterLink: ({ href, children }: { href: string; children: ReactNode }) =>
      create('a', { href }, children),
  };
});

import { RecentAssets } from './recent-assets';

describe('recent assets', () => {
  beforeEach(() => {
    state.buildEntryPath.mockClear();
    state.translate.mockClear();
  });

  it('shows the eight most recently updated assets with their entry paths', () => {
    const overview = createOverview({
      recent_folders: [createEntry('folder-2', 2, 'folder'), createEntry('folder-9', 9, 'folder')],
      recent_entries: [
        createEntry('file-7', 7),
        createEntry('file-1', 1),
        createEntry('file-8', 8),
        createEntry('file-4', 4),
        createEntry('file-6', 6),
        createEntry('file-5', 5),
        createEntry('folder-3', 3, 'folder'),
      ],
    });

    const html = renderToStaticMarkup(createElement(RecentAssets, { overview }));

    expect(state.translate).toHaveBeenCalledWith('file.recentAssets');
    expect(html).toContain('最近更新资产');
    expect(state.buildEntryPath.mock.calls.map(([asset]) => asset.id)).toEqual([
      'folder-9',
      'file-8',
      'file-7',
      'file-6',
      'file-5',
      'file-4',
      'folder-3',
      'folder-2',
    ]);
    expect(html).toContain('href="/dashboard/file-manager?entry=folder-9"');
    expect(html).not.toContain('file-1');
  });
});

function createOverview(
  assets: Pick<FileOverview, 'recent_entries' | 'recent_folders'>
): FileOverview {
  return {
    space_id: 'space-1',
    logical_asset_size: 0,
    managed_physical_usage: 0,
    recycle_bin_size: 0,
    temporary_upload_size: 0,
    deduplication_savings: 0,
    quota_bytes: 0,
    quota_reserved_bytes: 0,
    type_distribution: [],
    ...assets,
  };
}

function createEntry(id: string, day: number, type: FileEntry['type'] = 'file'): FileEntry {
  const updatedAt = `2026-01-${String(day).padStart(2, '0')}T00:00:00.000Z`;
  return {
    id,
    type,
    space_id: 'space-1',
    owner_user_id: 'user-1',
    parent_id: type === 'folder' ? null : 'folder-parent',
    name: id,
    size_bytes: 0,
    mime_type: null,
    created_at: updatedAt,
    updated_at: updatedAt,
    tags: [],
    properties: {},
    preview_supported: false,
    download_only: false,
  };
}
