export type QueryParams = Record<string, unknown>;

export const pageQuery = (page: number, pageSize: number, params: QueryParams = {}) =>
  compactParams({
    page: page + 1,
    page_size: pageSize,
    ...params,
  });

export function pageKey(endpoint: string, page: number, pageSize: number, params: QueryParams = {}) {
  return [endpoint, { params: pageQuery(page, pageSize, params) }] as const;
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
    Object.entries(params).filter(([, value]) => value !== '' && value !== null && value !== undefined)
  );
}
