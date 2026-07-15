import type { NavItemDataProps } from '../types';

import { isActiveLink } from 'minimal-shared/utils';

export function isNavItemActive(pathname: string, item: NavItemDataProps) {
  return isActiveLink(pathname, item.path, item.deepMatch ?? !!item.children);
}

export function isNavBranchActive(pathname: string, item: NavItemDataProps): boolean {
  if (isNavItemActive(pathname, item)) return true;

  return item.children?.some((child) => isNavBranchActive(pathname, child)) ?? false;
}
