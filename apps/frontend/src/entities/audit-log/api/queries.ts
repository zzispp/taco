import type { CursorPageRequest } from 'src/shared/api/pagination';
import type {
  LoginLog,
  LoginLogListQuery,
  OperationLogDetail,
  OperationLogSummary,
  OperationLogListQuery,
} from '../model/types';

import useSWR from 'swr';
import { useTranslation } from 'react-i18next';

import { fetcher } from 'src/shared/api/http-client';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { auditLogEndpoints } from './endpoints';

type DetailKey = readonly [string, string];

export function useOperationLogs(
  request: Readonly<{
    cursor: CursorPageRequest;
    query: OperationLogListQuery;
    enabled: boolean;
  }>
) {
  return useAuditCursor<OperationLogSummary>({
    ...request,
    endpoint: request.enabled ? auditLogEndpoints.operationLogs : null,
  });
}

export function useLoginLogs(
  request: Readonly<{
    cursor: CursorPageRequest;
    query: LoginLogListQuery;
    enabled: boolean;
  }>
) {
  return useAuditCursor<LoginLog>({
    ...request,
    endpoint: request.enabled ? auditLogEndpoints.loginLogs : null,
  });
}

export function useOperationLogDetail(id: string | null, enabled: boolean) {
  const language = useAuditLanguage();
  const key: DetailKey | null =
    id && enabled ? [auditLogEndpoints.operationLog(id), language] : null;
  return useSWR<OperationLogDetail>(key, fetchDetail, { revalidateOnFocus: false });
}

function useAuditCursor<T>(request: Readonly<AuditCursorRequest>) {
  const language = useAuditLanguage();
  return useCursorResource<T>({
    endpoint: request.endpoint ?? '',
    request: request.cursor,
    params: request.query,
    context: language,
  });
}

type AuditCursorRequest = {
  endpoint: string | null;
  cursor: CursorPageRequest;
  query: Record<string, unknown>;
};

function useAuditLanguage() {
  const { i18n } = useTranslation();
  return i18n.resolvedLanguage ?? i18n.language;
}

function fetchDetail([endpoint]: DetailKey) {
  return fetcher<OperationLogDetail>(endpoint);
}
