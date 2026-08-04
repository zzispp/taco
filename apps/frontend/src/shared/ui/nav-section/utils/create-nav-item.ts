import type { NavItemDataProps, NavItemOptionsProps } from '../types';

import { cloneElement } from 'react';

import { RouterLink } from 'src/shared/routes/components';

// ----------------------------------------------------------------------

type CreateNavItemReturn = {
  subItem: boolean;
  rootItem: boolean;
  subDeepItem: boolean;
  baseProps: Record<string, any>;
  renderIcon: React.ReactNode;
  renderInfo: React.ReactNode;
};

type CreateNavItemProps = Pick<NavItemDataProps, 'path' | 'icon' | 'info'> & NavItemOptionsProps;

export function createNavItem({
  path,
  icon,
  info,
  depth,
  render,
  hasChild,
  externalLink,
  enabledRootRedirect,
}: CreateNavItemProps): CreateNavItemReturn {
  const rootItem = depth === 1;
  const subItem = !rootItem;
  const subDeepItem = Number(depth) > 2;

  return {
    subItem,
    rootItem,
    subDeepItem,
    baseProps: createBaseProps({ path, hasChild, externalLink, enabledRootRedirect }),
    renderIcon: resolveNavIcon(icon, render),
    renderInfo: resolveNavInfo(info, render),
  };
}

function createBaseProps({
  path,
  hasChild,
  externalLink,
  enabledRootRedirect,
}: Pick<CreateNavItemProps, 'path' | 'hasChild' | 'externalLink' | 'enabledRootRedirect'>) {
  if (hasChild && !enabledRootRedirect) {
    return { component: 'div' };
  }

  return externalLink
    ? { href: path, target: '_blank', rel: 'noopener noreferrer' }
    : { component: RouterLink, href: path };
}

function resolveNavIcon(icon: NavItemDataProps['icon'], render: NavItemOptionsProps['render']) {
  if (icon && render?.navIcon && typeof icon === 'string') {
    return render.navIcon[icon];
  }

  return icon;
}

function resolveNavInfo(info: NavItemDataProps['info'], render: NavItemOptionsProps['render']) {
  if (!info || !render?.navInfo || !Array.isArray(info)) {
    return info;
  }

  const [key, value] = info;
  const element = render.navInfo(value)[key];

  return element ? cloneElement(element) : null;
}
