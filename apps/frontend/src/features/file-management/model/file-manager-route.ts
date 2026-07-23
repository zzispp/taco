import type { FileEntry } from 'src/entities/file';

import { paths } from 'src/shared/routes/paths';

const FILE_MANAGER_ROUTE_QUERY = {
  spaceId: 'space_id',
  parentId: 'parent_id',
  detailId: 'detail_id',
} as const;

export type FileManagerRoute = Readonly<{
  spaceId: string | null;
  parentId: string | null;
  detailId: string | null;
}>;

export type FileManagerRouteSearchParams = Pick<URLSearchParams, 'get'>;

export const EMPTY_FILE_MANAGER_ROUTE = Object.freeze({
  spaceId: null,
  parentId: null,
  detailId: null,
}) satisfies FileManagerRoute;

export function buildFileManagerEntryPath(
  entry: Pick<FileEntry, 'id' | 'space_id' | 'parent_id' | 'type'>
): string {
  return buildFileManagerRoutePath({
    spaceId: entry.space_id,
    parentId: entry.type === 'folder' ? entry.id : entry.parent_id,
    detailId: entry.type === 'file' ? entry.id : null,
  });
}

export function buildFileManagerRoutePath(route: FileManagerRoute): string {
  const query = new URLSearchParams();
  appendRouteValue(query, FILE_MANAGER_ROUTE_QUERY.spaceId, route.spaceId);
  appendRouteValue(query, FILE_MANAGER_ROUTE_QUERY.parentId, route.parentId);
  appendRouteValue(query, FILE_MANAGER_ROUTE_QUERY.detailId, route.detailId);
  const queryString = query.toString();
  return queryString
    ? `${paths.dashboard.fileManager}?${queryString}`
    : paths.dashboard.fileManager;
}

export function parseFileManagerRoute(
  searchParams: FileManagerRouteSearchParams
): FileManagerRoute {
  return {
    spaceId: routeValue(searchParams.get(FILE_MANAGER_ROUTE_QUERY.spaceId)),
    parentId: routeValue(searchParams.get(FILE_MANAGER_ROUTE_QUERY.parentId)),
    detailId: routeValue(searchParams.get(FILE_MANAGER_ROUTE_QUERY.detailId)),
  };
}

export function fileManagerRouteKey(route: FileManagerRoute): string {
  return JSON.stringify([
    routeValue(route.spaceId),
    routeValue(route.parentId),
    routeValue(route.detailId),
  ]);
}

function appendRouteValue(query: URLSearchParams, key: string, value: string | null) {
  const normalizedValue = routeValue(value);
  if (normalizedValue) query.set(key, normalizedValue);
}

function routeValue(value: string | null): string | null {
  const normalizedValue = value?.trim();
  return normalizedValue || null;
}
