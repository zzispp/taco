import type { Menu } from '../model/types';
import type { QueryParams } from 'src/shared/api/pagination';

import { useCursorCollection } from 'src/shared/api/use-cursor-collection';

import { menuEndpoints } from './endpoints';

export function useMenus(params: QueryParams = {}) {
  return useCursorCollection<Menu>({ endpoint: menuEndpoints.menus, params });
}
