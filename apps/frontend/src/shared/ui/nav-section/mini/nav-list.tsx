'use client';

import type { NavListProps, NavSubListProps, NavItemDataProps } from '../types';

import { useEffect, useCallback } from 'react';
import { usePopoverHover } from 'minimal-shared/hooks';
import { isActiveLink, isExternalLink } from 'minimal-shared/utils';

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
  const pathname = usePathname();
  const isActive = isActiveLink(pathname, data.path, data.deepMatch ?? !!data.children);
  const {
    open,
    onOpen,
    onClose,
    anchorEl,
    elementRef: navItemRef,
  } = usePopoverHover<HTMLButtonElement>();
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

  return { open, onClose, anchorEl, navItemRef, isActive, id, handleOpenMenu };
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
      path={data.path}
      icon={data.icon}
      info={data.info}
      title={data.title}
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

  return (
    <NavDropdown
      disableScrollLock
      aria-hidden={!menu.open}
      id={menu.id}
      open={menu.open}
      anchorEl={menu.anchorEl}
      anchorOrigin={{ vertical: 'center', horizontal: 'left' }}
      transformOrigin={{ vertical: 'center', horizontal: 'right' }}
      slotProps={{
        paper: {
          onMouseEnter: menu.handleOpenMenu,
          onMouseLeave: menu.onClose,
          className: navSectionClasses.dropdown.root,
        },
      }}
      sx={{ ...cssVars }}
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
