import type { OnlineSessionQuery, OnlineSessionsResponse } from '../model/types';

import useSWR from 'swr';

import { fetcher } from 'src/shared/api/http-client';
import { compactParams } from 'src/shared/api/pagination';

import { onlineSessionEndpoints } from './endpoints';

const swrOptions = {
  keepPreviousData: true,
  revalidateOnFocus: false,
};

export function useOnlineSessions(filters: OnlineSessionQuery) {
  const { data, error, isLoading, isValidating } = useSWR<OnlineSessionsResponse>(
    [onlineSessionEndpoints.list, { params: compactParams(filters) }],
    fetcher,
    swrOptions
  );

  return {
    rows: data?.rows ?? [],
    total: data?.total ?? 0,
    error,
    isLoading,
    isValidating,
  };
}
