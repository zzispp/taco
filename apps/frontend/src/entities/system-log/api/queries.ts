import type { CursorPageRequest } from 'src/shared/api/pagination';
import type {
  SystemLogDetail,
  SystemLogSummary,
  SystemLogFilterQuery,
  SystemLogCleanupExecution,
} from '../model/types';

import useSWR from 'swr';
import { useTranslation } from 'react-i18next';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { systemLogEndpoints } from './endpoints';

type DetailKey = readonly [string, string];

export function useSystemLogs(
  request: Readonly<{ cursor: CursorPageRequest; query: SystemLogFilterQuery; enabled: boolean }>
) {
  const { i18n } = useTranslation();
  return useCursorResource<SystemLogSummary>({
    endpoint: request.enabled ? systemLogEndpoints.logs : '',
    request: request.cursor,
    params: request.query,
    context: i18n.resolvedLanguage ?? i18n.language,
  });
}

export function useSystemLogDetail(id: string | null, enabled: boolean) {
  const { i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const key: DetailKey | null = id && enabled ? [systemLogEndpoints.detail(id), language] : null;
  return useSWR<SystemLogDetail>(key, fetchDetail, { revalidateOnFocus: false });
}

export function useSystemLogCleanupExecution(id: string | null, enabled: boolean) {
  const { i18n } = useTranslation();
  const language = i18n.resolvedLanguage ?? i18n.language;
  const key: DetailKey | null =
    id && enabled ? [systemLogEndpoints.cleanExecution(id), language] : null;
  return useSWR<SystemLogCleanupExecution>(key, fetchCleanupExecution, {
    refreshInterval: (execution) =>
      execution && isTerminalCleanupExecution(execution.state) ? 0 : 1_000,
    revalidateOnFocus: false,
  });
}

function fetchDetail([endpoint]: DetailKey) {
  return fetcher<SystemLogDetail>(endpoint);
}

function fetchCleanupExecution([endpoint]: DetailKey) {
  return fetcher<SystemLogCleanupExecution>(endpoint);
}

function isTerminalCleanupExecution(state: SystemLogCleanupExecution['state']) {
  return !['pending', 'running'].includes(state);
}
