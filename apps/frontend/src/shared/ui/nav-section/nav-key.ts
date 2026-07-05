import type { NavSectionProps, NavItemDataProps } from './types';

type NavSectionGroup = NavSectionProps['data'][number];

export function navGroupKey(group: NavSectionGroup, index: number): string {
  const firstItem = group.items[0];
  return ['group', group.code, firstItem?.code, firstItem?.path, group.subheader, index]
    .filter(Boolean)
    .join(':');
}

export function navItemKey(item: NavItemDataProps, index: number): string {
  return ['item', item.code, item.path, item.title, index].filter(Boolean).join(':');
}
