'use client';

import type { FileSpace, FileSpaceListQuery } from 'src/entities/file';

import { useMemo, useState, useEffect, useCallback } from 'react';

import { useTable, DEFAULT_TABLE_LIMIT } from 'src/shared/ui/table';

import { useFileSpaces } from 'src/entities/file';

type FileSpaceSelectorOptions = Readonly<{
  selectedSpaceId: string | undefined;
  enabled: boolean;
}>;

export function useFileSpaceSelector({ selectedSpaceId, enabled }: FileSpaceSelectorOptions) {
  const [search, setSearch] = useState('');
  const table = useTable({
    defaultLimit: DEFAULT_TABLE_LIMIT,
    defaultOrderBy: 'owner_name',
    scopeKey: search.trim(),
  });
  const query = useMemo(() => fileSpaceSelectorQuery(search), [search]);
  const spaces = useFileSpaces(table.cursorRequest, query, enabled);
  const selection = useRememberedFileSpace(selectedSpaceId, spaces.items);
  return { selectedSpaceId, search, setSearch, table, spaces, ...selection };
}

export type FileSpaceSelectorState = ReturnType<typeof useFileSpaceSelector>;

export function fileSpaceSelectorQuery(search: string): FileSpaceListQuery {
  const normalizedSearch = search.trim();
  return {
    ...(normalizedSearch ? { search: normalizedSearch } : {}),
    sort_by: 'owner_name',
    sort_order: 'asc',
  };
}

function useRememberedFileSpace(selectedSpaceId: string | undefined, spaces: readonly FileSpace[]) {
  const [remembered, setRemembered] = useState<FileSpace | null>(null);
  const current = useMemo(
    () => spaces.find((space) => space.id === selectedSpaceId) ?? null,
    [selectedSpaceId, spaces]
  );
  useEffect(() => {
    if (!selectedSpaceId) {
      setRemembered(null);
      return;
    }
    if (current) setRemembered(current);
  }, [current, selectedSpaceId]);
  const selectedSpace = remembered?.id === selectedSpaceId ? remembered : current;
  const rememberSpace = useCallback((space: FileSpace | null) => setRemembered(space), []);
  return { selectedSpace, rememberSpace };
}
