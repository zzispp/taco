import type { CursorPageRequest } from 'src/shared/api/pagination';
import type { OnlineSession, OnlineSessionQuery } from '../model/types';

import { cursorKey } from 'src/shared/api/pagination';
import { useCursorResource } from 'src/shared/api/use-cursor-resource';

import { onlineSessionEndpoints } from './endpoints';

export function onlineSessionsKey(request: CursorPageRequest, filters: OnlineSessionQuery) {
  return cursorKey(onlineSessionEndpoints.list, request, filters);
}

export function useOnlineSessions(request: CursorPageRequest, filters: OnlineSessionQuery) {
  const resource = useCursorResource<OnlineSession>({
    endpoint: onlineSessionEndpoints.list,
    request,
    params: filters,
  });
  return { ...resource, rows: resource.items };
}
