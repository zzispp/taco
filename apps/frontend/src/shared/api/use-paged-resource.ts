import type { PageResponse } from './types';

import useSWR from 'swr';
import { useMemo } from 'react';

import { pageKey } from './pagination';
import { fetcher } from './http-client';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function usePagedResource<T>(endpoint: string, page: number, pageSize: number) {
  const { data, isLoading, error, isValidating } = useSWR<PageResponse<T>>(
    pageKey(endpoint, page, pageSize),
    fetcher,
    swrOptions
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

export const stableSWRConfig = swrOptions;
