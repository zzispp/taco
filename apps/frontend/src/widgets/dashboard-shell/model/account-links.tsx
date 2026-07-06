import type { NavSectionProps } from 'src/shared/ui/nav-section';
import type { AccountLink } from 'src/widgets/dashboard-shell/ui/account-drawer';

import { paths } from 'src/shared/routes/paths';
import { Iconify } from 'src/shared/ui/iconify';

import { NAV_ICONS } from 'src/entities/menu';

export function accountLinksFromNavData(
  data: NavSectionProps['data'],
  profileLabel: string
): AccountLink[] {
  return [profileLink(profileLabel), ...data.flatMap((section) => flattenItems(section.items))];
}

function profileLink(label: string): AccountLink {
  return {
    key: 'account-profile',
    label,
    href: paths.dashboard.profile,
    icon: <Iconify icon="solar:user-id-bold" />,
  };
}

function flattenItems(items: NavItem[], depth = 0): AccountLink[] {
  return items.flatMap((item) => [
    accountLink(item, depth),
    ...flattenItems(item.children ?? [], depth + 1),
  ]);
}

function accountLink(item: NavItem, depth: number): AccountLink {
  const icon = typeof item.icon === 'string' ? NAV_ICONS?.[item.icon] : item.icon;
  return {
    key: item.code ?? item.path,
    label: item.title,
    href: item.path,
    icon: icon ?? <Iconify icon="solar:forward-bold" />,
    depth,
  };
}

type NavItem = NavSectionProps['data'][number]['items'][number];
