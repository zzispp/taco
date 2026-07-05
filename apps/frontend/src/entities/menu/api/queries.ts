import type { Menu } from '../model/types';
import type { QueryParams } from 'src/shared/api/pagination';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { menuEndpoints } from './endpoints';

export function useMenus(page: number, pageSize: number, params: QueryParams = {}) {
  return usePagedResource<Menu>(menuEndpoints.menus, page, pageSize, params);
}
