import type { PageResponse } from './types';
import type { QueryParams } from './pagination';

import useSWR from 'swr';
import { useMemo } from 'react';

import { pageKey } from './pagination';
import { fetcher } from './http-client';

const defaultSWRConfig = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export type PagedResourceOptions = Readonly<{
  endpoint: string;
  page: number;
  pageSize: number;
  params?: QueryParams;
  keepPreviousData?: boolean;
}>;

export type PagedResourceState<T> = Readonly<{
  data: PageResponse<T> | undefined;
  items: T[];
  total: number;
  isLoading: boolean;
  error: unknown;
  isValidating: boolean;
}>;

export function usePagedResource<T>({
  endpoint,
  page,
  pageSize,
  params = {},
  keepPreviousData = true,
}: PagedResourceOptions): PagedResourceState<T> {
  const { data, isLoading, error, isValidating } = useSWR<PageResponse<T>>(
    pagedResourceKey({ endpoint, page, pageSize, params, keepPreviousData }),
    fetcher,
    pagedResourceConfig(keepPreviousData)
  );

  return useMemo(
    () => ({
      data,
      items: data?.items ?? [],
      total: data?.total ?? 0,
      isLoading,
      error,
      isValidating,
    }),
    [data, error, isLoading, isValidating]
  );
}

export function pagedResourceKey(options: PagedResourceOptions) {
  return options.endpoint
    ? pageKey(options.endpoint, options.page, options.pageSize, options.params ?? {})
    : null;
}

export function pagedResourceConfig(keepPreviousData = true) {
  return { ...defaultSWRConfig, keepPreviousData };
}

export const stableSWRConfig = defaultSWRConfig;
