export const pageQuery = (page: number, pageSize: number) => ({
  page: page + 1,
  page_size: pageSize,
});

export function pageKey(endpoint: string, page: number, pageSize: number) {
  return [endpoint, { params: pageQuery(page, pageSize) }] as const;
}

export async function requestData<T>(request: Promise<{ data: T }>) {
  const response = await request;
  return response.data;
}

export function isEndpointKey(key: unknown, endpoint: string) {
  return key === endpoint || (Array.isArray(key) && key[0] === endpoint);
}
