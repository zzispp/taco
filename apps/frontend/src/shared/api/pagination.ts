export type QueryParams = Record<string, unknown>;

export const DEFAULT_CURSOR_LIMIT = 20;
export const CURSOR_LIMIT_OPTIONS = [20, 50, 100] as const;

export type CursorPageRequest = Readonly<{
  limit: number;
  cursor?: string;
}>;

export const cursorQuery = (request: CursorPageRequest, params: QueryParams = {}) =>
  compactParams({
    ...params,
    limit: request.limit,
    cursor: request.cursor,
  });

export function cursorKey(endpoint: string, request: CursorPageRequest, params: QueryParams = {}) {
  return [endpoint, { params: cursorQuery(request, params) }] as const;
}

export async function requestData<T>(request: Promise<{ data: T }>) {
  const response = await request;
  return response.data;
}

export function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}

export function compactParams(params: QueryParams) {
  return Object.fromEntries(
    Object.entries(params).filter(
      ([, value]) => value !== '' && value !== null && value !== undefined
    )
  );
}
