import { createElement } from 'react';
import { it, vi, expect, describe } from 'vitest';
import { renderToStaticMarkup } from 'react-dom/server';

const { fetcherMock, useSWRMock } = vi.hoisted(() => ({
  fetcherMock: vi.fn(),
  useSWRMock: vi.fn((_key: unknown, _fetcher?: unknown, _options?: unknown) => ({
    data: undefined,
    mutate: vi.fn(),
  })),
}));

vi.mock('swr', () => ({ default: useSWRMock }));
vi.mock('src/shared/api/http-client', () => ({ fetcher: fetcherMock }));

import {
  useFileSpaces,
  useFileEntries,
  useFileOverview,
  useFileProviders,
  useFileDirectoryTrail,
  isFileDirectoryTrailKey,
} from './queries';

describe('file entity queries', () => {
  it('matches only directory-trail cache keys for file-resource refreshes', () => {
    expect(isFileDirectoryTrailKey(['file-directory-trail', 'folder-1'])).toBe(true);
    expect(isFileDirectoryTrailKey(['file-directory-trail', 1])).toBe(false);
    expect(isFileDirectoryTrailKey(['file-directory-trail', 'folder-1', 'unexpected'])).toBe(false);
    expect(isFileDirectoryTrailKey('/api/system/files/folder-1')).toBe(false);
  });

  it('keys an overview request by the selected space', () => {
    renderHookProbe(() => useFileOverview('space/managed'));

    expect(useSWRMock.mock.calls[0]?.[0]).toBe(
      '/api/system/files/overview?space_id=space%2Fmanaged'
    );
  });

  it('disables overview and provider requests without permission', () => {
    renderHookProbe(() => {
      useFileOverview('space-1', false);
      useFileProviders(false);
    });

    expect(useSWRMock.mock.calls.map((call) => call[0])).toEqual([null, null]);
  });

  it('uses the provider summaries endpoint when enabled', () => {
    renderHookProbe(() => useFileProviders());

    expect(useSWRMock.mock.calls[0]?.[0]).toBe('/api/system/file-providers');
  });

  it('loads a directory trail through one list-authorized endpoint request', async () => {
    renderHookProbe(() => useFileDirectoryTrail('folder/managed'));

    expect(useSWRMock.mock.calls[0]?.[0]).toEqual(['file-directory-trail', 'folder/managed']);
    const loader = useSWRMock.mock.calls[0]?.[1] as (
      key: readonly ['file-directory-trail', string]
    ) => Promise<unknown>;
    await loader(['file-directory-trail', 'folder/managed']);

    expect(fetcherMock).toHaveBeenCalledWith('/api/system/files/folder%2Fmanaged/directory-trail');
  });

  it('loads one server-filtered page of file spaces instead of a complete collection', () => {
    renderHookProbe(() =>
      useFileSpaces({ limit: 20 }, { search: 'platform', sort_by: 'owner_name', sort_order: 'asc' })
    );

    expect(useSWRMock.mock.calls[0]?.[0]).toEqual([
      '/api/system/file-spaces',
      {
        params: {
          limit: 20,
          search: 'platform',
          sort_by: 'owner_name',
          sort_order: 'asc',
        },
      },
      '{"search":"platform","sort_by":"owner_name","sort_order":"asc"}',
    ]);
  });

  it('loads only folder children for a move destination page', () => {
    renderHookProbe(() =>
      useFileEntries(
        { limit: 20 },
        {
          space_id: 'space-1',
          parent_id: '00000000-0000-0000-0000-000000000000',
          kind: 'folder',
          trashed: false,
          sort_by: 'name',
          sort_order: 'asc',
        }
      )
    );

    expect(useSWRMock.mock.calls[0]?.[0]).toEqual([
      '/api/system/files',
      {
        params: {
          limit: 20,
          space_id: 'space-1',
          parent_id: '00000000-0000-0000-0000-000000000000',
          kind: 'folder',
          trashed: false,
          sort_by: 'name',
          sort_order: 'asc',
        },
      },
      '{"space_id":"space-1","parent_id":"00000000-0000-0000-0000-000000000000","kind":"folder","trashed":false,"sort_by":"name","sort_order":"asc"}',
    ]);
  });

  it('does not issue a file-space request without list permission', () => {
    renderHookProbe(() => useFileSpaces({ limit: 20 }, {}, false));

    expect(useSWRMock.mock.calls[0]?.[0]).toBeNull();
  });
});

function renderHookProbe(run: () => void) {
  useSWRMock.mockClear();
  fetcherMock.mockClear();
  function Probe() {
    run();
    return null;
  }
  renderToStaticMarkup(createElement(Probe));
}
