import type { AxiosRequestConfig } from 'axios';
import type { CursorPageResponse } from './types';
import type { QueryParams, CursorPageRequest } from './pagination';

import useSWR from 'swr';
import { useMemo } from 'react';

import { fetcher } from './http-client';
import { cursorQuery } from './pagination';

const defaultSWRConfig = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export type CursorResourceOptions = Readonly<{
  endpoint: string;
  request: CursorPageRequest;
  params?: QueryParams;
  context?: string;
  keepPreviousData?: boolean;
}>;

export type CursorResourceState<T> = Readonly<{
  data: CursorPageResponse<T> | undefined;
  items: T[];
  itemCount: number;
  nextCursor: string | null;
  previousCursor: string | null;
  hasNext: boolean;
  hasPrevious: boolean;
  isLoading: boolean;
  error: unknown;
  isValidating: boolean;
}>;

type CursorResourceKey = readonly [string, AxiosRequestConfig, string];

export function useCursorResource<T>({
  endpoint,
  request,
  params = {},
  context = '',
  keepPreviousData = true,
}: CursorResourceOptions): CursorResourceState<T> {
  const { data, isLoading, error, isValidating } = useSWR<CursorPageResponse<T>>(
    cursorResourceKey({ endpoint, request, params, context, keepPreviousData }),
    fetchCursorPage,
    cursorResourceConfig(keepPreviousData)
  );

  return useMemo(
    () => cursorResourceState(data, { isLoading, error, isValidating }),
    [data, error, isLoading, isValidating]
  );
}

export function cursorResourceKey(options: CursorResourceOptions): CursorResourceKey | null {
  if (!options.endpoint) return null;
  return [
    options.endpoint,
    { params: cursorQuery(options.request, options.params ?? {}) },
    options.context ?? '',
  ];
}

export function cursorResourceConfig(keepPreviousData = true) {
  return { ...defaultSWRConfig, keepPreviousData };
}

export const stableSWRConfig = defaultSWRConfig;

function fetchCursorPage<T>([endpoint, config]: CursorResourceKey) {
  return fetcher<CursorPageResponse<T>>([endpoint, config]);
}

function cursorResourceState<T>(
  data: CursorPageResponse<T> | undefined,
  state: Pick<CursorResourceState<T>, 'isLoading' | 'error' | 'isValidating'>
): CursorResourceState<T> {
  if (!data) {
    return {
      data: undefined,
      items: [],
      itemCount: 0,
      nextCursor: null,
      previousCursor: null,
      hasNext: false,
      hasPrevious: false,
      ...state,
    };
  }
  return {
    data,
    items: data.items,
    itemCount: data.items.length,
    nextCursor: data.next_cursor,
    previousCursor: data.previous_cursor,
    hasNext: data.has_next,
    hasPrevious: data.has_previous,
    ...state,
  };
}
