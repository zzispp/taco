import type { AxiosRequestConfig } from 'axios';
import type { QueryParams } from './pagination';
import type { CursorPageResponse } from './types';

import useSWRInfinite from 'swr/infinite';
import { useEffect, useCallback } from 'react';

import { fetcher } from './http-client';
import { cursorQuery, CURSOR_LIMIT_OPTIONS } from './pagination';

const COLLECTION_LIMIT = CURSOR_LIMIT_OPTIONS[CURSOR_LIMIT_OPTIONS.length - 1];

type CollectionKey = readonly [string, AxiosRequestConfig, string];

export type CursorCollectionOptions = Readonly<{
  endpoint: string;
  params?: QueryParams;
  context?: string;
}>;

export function useCursorCollection<T>({
  endpoint,
  params = {},
  context = '',
}: CursorCollectionOptions) {
  const getKey = (batchIndex: number, previous: CursorPageResponse<T> | null) =>
    cursorCollectionKey({ endpoint, params, context }, batchIndex, previous);
  const resource = useSWRInfinite<CursorPageResponse<T>>(getKey, fetchCollectionBatch, {
    revalidateOnFocus: false,
    revalidateFirstPage: false,
  });
  const { data, size, isLoading, setSize, mutate } = resource;
  const collection = collectionData(data);
  const nextCursor = cursorCollectionNextCursor(collection.lastBatch);
  const reset = useCallback(async () => {
    await setSize(1);
    await mutate();
  }, [mutate, setSize]);

  useEffect(() => {
    if (nextCursor) {
      void setSize((currentSize) => currentSize + 1);
    }
  }, [nextCursor, setSize]);

  return {
    ...resource,
    reset,
    items: collection.items,
    isLoading: isLoading || size > collection.batchCount || nextCursor !== null,
  };
}

export function cursorCollectionShouldLoadNext<T>(batch: CursorPageResponse<T> | undefined) {
  return cursorCollectionNextCursor(batch) !== null;
}

export function cursorCollectionKey<T>(
  options: CursorCollectionOptions,
  batchIndex: number,
  previous: CursorPageResponse<T> | null
): CollectionKey | null {
  if (!options.endpoint) return null;
  if (batchIndex > 0) {
    if (!previous?.has_next) return null;
    if (!previous.next_cursor) {
      throw new Error('Cursor collection response has_next without next_cursor');
    }
  }
  const request = {
    limit: COLLECTION_LIMIT,
    ...(previous?.next_cursor ? { cursor: previous.next_cursor } : {}),
  };
  return [
    options.endpoint,
    { params: cursorQuery(request, options.params ?? {}) },
    options.context ?? '',
  ];
}

function fetchCollectionBatch<T>([endpoint, config]: CollectionKey) {
  return fetcher<CursorPageResponse<T>>([endpoint, config]);
}

function cursorCollectionNextCursor<T>(batch: CursorPageResponse<T> | undefined) {
  if (!batch || !batch.has_next) return null;
  if (!batch.next_cursor) {
    throw new Error('Cursor collection response has_next without next_cursor');
  }
  return batch.next_cursor;
}

function collectionData<T>(data: CursorPageResponse<T>[] | undefined) {
  if (!data) return { items: [], batchCount: 0, lastBatch: undefined };
  return {
    items: data.flatMap((batch) => batch.items),
    batchCount: data.length,
    lastBatch: data.at(-1),
  };
}
