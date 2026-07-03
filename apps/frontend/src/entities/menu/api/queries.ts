import type { MenuItem, MenuSection } from '../model/types';

import { usePagedResource } from 'src/shared/api/use-paged-resource';

import { menuEndpoints } from './endpoints';

export function useMenuSections(page: number, pageSize: number) {
  return usePagedResource<MenuSection>(menuEndpoints.menuSections, page, pageSize);
}

export function useMenuItems(page: number, pageSize: number) {
  return usePagedResource<MenuItem>(menuEndpoints.menuItems, page, pageSize);
}
