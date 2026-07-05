import type { PageResponse } from './types';
import type { QueryParams } from './pagination';

import useSWR from 'swr';
import { useMemo } from 'react';

import { pageKey } from './pagination';
import { fetcher } from './http-client';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function usePagedResource<T>(
  endpoint: string,
  page: number,
  pageSize: number,
  params: QueryParams = {}
) {
  const { data, isLoading, error, isValidating } = useSWR<PageResponse<T>>(
    endpoint ? pageKey(endpoint, page, pageSize, params) : null,
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
