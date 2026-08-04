'use client';

import type { NavListProps, NavSubListProps, NavItemDataProps } from '../types';

import { useEffect, useCallback } from 'react';
import { usePopoverHover } from 'minimal-shared/hooks';
import { isActiveLink, isExternalLink } from 'minimal-shared/utils';

import { useTheme } from '@mui/material/styles';
import { popoverClasses } from '@mui/material/Popover';

import { usePathname } from 'src/shared/routes/hooks';

import { NavItem } from './nav-item';
import { navItemKey } from '../nav-key';
import { navSectionClasses } from '../styles';
import { NavUl, NavLi, NavDropdown, NavDropdownPaper } from '../components';

// ----------------------------------------------------------------------

type NavMenuState = ReturnType<typeof useNavMenu>;

export function NavList(props: NavListProps) {
  const { data, checkPermissions } = props;
  const menu = useNavMenu(data);

  if (data.allowedRoles && checkPermissions && checkPermissions(data.allowedRoles)) {
    return null;
  }

  return (
    <NavLi disabled={data.disabled}>
      <NavListItem {...props} menu={menu} />
      <NavListDropdown {...props} menu={menu} />
    </NavLi>
  );
}

function useNavMenu(data: NavItemDataProps) {
  const theme = useTheme();
  const pathname = usePathname();
  const isActive = isActiveLink(pathname, data.path, data.deepMatch ?? !!data.children);
  const {
    open,
    onOpen,
    onClose,
    anchorEl,
    elementRef: navItemRef,
  } = usePopoverHover<HTMLButtonElement>();
  const isRtl = theme.direction === 'rtl';
  const id = open ? `${data.title}-popover` : undefined;

  useEffect(() => {
    if (open) {
      onClose();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname]);

  const handleOpenMenu = useCallback(() => {
    if (data.children) {
      onOpen();
    }
  }, [data.children, onOpen]);

  return { open, onClose, anchorEl, navItemRef, isActive, isRtl, id, handleOpenMenu };
}

type NavListItemProps = NavListProps & { menu: NavMenuState };

function NavListItem({
  data,
  depth,
  render,
  slotProps,
  enabledRootRedirect,
  menu,
}: NavListItemProps) {
  return (
    <NavItem
      ref={menu.navItemRef}
      aria-describedby={menu.id}
      title={data.title}
      path={data.path}
      icon={data.icon}
      info={data.info}
      caption={data.caption}
      active={menu.isActive}
      open={menu.open}
      disabled={data.disabled}
      depth={depth}
      render={render}
      hasChild={!!data.children}
      externalLink={isExternalLink(data.path)}
      enabledRootRedirect={enabledRootRedirect}
      slotProps={depth === 1 ? slotProps?.rootItem : slotProps?.subItem}
      onMouseEnter={menu.handleOpenMenu}
      onMouseLeave={menu.onClose}
    />
  );
}

type NavListDropdownProps = NavListProps & { menu: NavMenuState };

function NavListDropdown({
  data,
  depth,
  render,
  cssVars,
  slotProps,
  checkPermissions,
  enabledRootRedirect,
  menu,
}: NavListDropdownProps) {
  if (!data.children) {
    return null;
  }

  const { anchorOrigin, transformOrigin } = getPopoverOrigins(depth, menu.isRtl);

  return (
    <NavDropdown
      disableScrollLock
      aria-hidden={!menu.open}
      id={menu.id}
      open={menu.open}
      anchorEl={menu.anchorEl}
      anchorOrigin={anchorOrigin}
      transformOrigin={transformOrigin}
      slotProps={{
        paper: {
          onMouseEnter: menu.handleOpenMenu,
          onMouseLeave: menu.onClose,
          className: navSectionClasses.dropdown.root,
        },
      }}
      sx={getDropdownSx(cssVars, depth)}
    >
      <NavDropdownPaper
        className={navSectionClasses.dropdown.paper}
        sx={slotProps?.dropdown?.paper}
      >
        <NavSubList
          data={data.children}
          depth={depth}
          render={render}
          cssVars={cssVars}
          slotProps={slotProps}
          checkPermissions={checkPermissions}
          enabledRootRedirect={enabledRootRedirect}
        />
      </NavDropdownPaper>
    </NavDropdown>
  );
}

function getPopoverOrigins(depth: number | undefined, isRtl: boolean) {
  return depth === 1
    ? ({
        anchorOrigin: { vertical: 'bottom', horizontal: isRtl ? 'right' : 'left' },
        transformOrigin: { vertical: 'top', horizontal: isRtl ? 'right' : 'left' },
      } as const)
    : ({
        anchorOrigin: { vertical: 'center', horizontal: isRtl ? 'left' : 'right' },
        transformOrigin: { vertical: 'center', horizontal: isRtl ? 'right' : 'left' },
      } as const);
}

function getDropdownSx(cssVars: NavListProps['cssVars'], depth: number | undefined) {
  return {
    ...cssVars,
    [`& .${popoverClasses.paper}`]: {
      ...(depth === 1 && { pt: 1, ml: -0.75 }),
    },
  };
}

// ----------------------------------------------------------------------

function NavSubList({
  data,
  render,
  cssVars,
  depth = 0,
  slotProps,
  checkPermissions,
  enabledRootRedirect,
}: NavSubListProps) {
  return (
    <NavUl sx={{ gap: 0.5 }}>
      {data.map((list, index) => (
        <NavList
          key={navItemKey(list, index)}
          data={list}
          render={render}
          depth={depth + 1}
          cssVars={cssVars}
          slotProps={slotProps}
          checkPermissions={checkPermissions}
          enabledRootRedirect={enabledRootRedirect}
        />
      ))}
    </NavUl>
  );
}
