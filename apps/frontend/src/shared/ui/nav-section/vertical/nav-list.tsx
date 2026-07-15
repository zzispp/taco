'use client';

import type { NavListProps, NavSubListProps } from '../types';

import { useBoolean } from 'minimal-shared/hooks';
import { isExternalLink } from 'minimal-shared/utils';
import { useRef, useEffect, useCallback, type RefObject } from 'react';

import { usePathname } from 'src/shared/routes/hooks';

import { NavItem } from './nav-item';
import { navItemKey } from '../nav-key';
import { navSectionClasses } from '../styles';
import { NavUl, NavLi, NavCollapse } from '../components';
import { isNavItemActive, isNavBranchActive } from '../utils';

// ----------------------------------------------------------------------

export function NavList(props: NavListProps) {
  const { data, checkPermissions } = props;
  const pathname = usePathname();
  const navItemRef = useRef<HTMLButtonElement>(null);

  const isActive = isNavItemActive(pathname, data);
  const branchActive = isNavBranchActive(pathname, data);

  const { value: open, setValue: setOpen, onToggle } = useBoolean(branchActive);

  useEffect(() => {
    setOpen(branchActive);
  }, [branchActive, pathname, setOpen]);

  const handleToggleMenu = useCallback(() => {
    if (data.children) {
      onToggle();
    }
  }, [data.children, onToggle]);

  // Hidden item by role
  if (data.allowedRoles && checkPermissions && checkPermissions(data.allowedRoles)) {
    return null;
  }

  return (
    <NavListContent
      {...props}
      active={isActive}
      open={open}
      navItemRef={navItemRef}
      onClick={handleToggleMenu}
    />
  );
}

// ----------------------------------------------------------------------

type NavListContentProps = NavListProps & {
  active: boolean;
  open: boolean;
  navItemRef: RefObject<HTMLButtonElement | null>;
  onClick: () => void;
};

function NavListContent({
  data,
  depth,
  render,
  slotProps,
  active,
  open,
  navItemRef,
  checkPermissions,
  enabledRootRedirect,
  onClick,
}: NavListContentProps) {
  return (
    <NavLi
      disabled={data.disabled}
      sx={{
        ...(!!data.children && {
          [`& .${navSectionClasses.li}`]: {
            '&:first-of-type': { mt: 'var(--nav-item-gap)' },
          },
        }),
      }}
    >
      <NavListItem
        data={data}
        depth={depth}
        render={render}
        slotProps={slotProps}
        open={open}
        active={active}
        navItemRef={navItemRef}
        enabledRootRedirect={enabledRootRedirect}
        onClick={onClick}
      />
      <NavListCollapse
        data={data}
        depth={depth}
        render={render}
        slotProps={slotProps}
        open={open}
        checkPermissions={checkPermissions}
        enabledRootRedirect={enabledRootRedirect}
      />
    </NavLi>
  );
}

// ----------------------------------------------------------------------

type NavListItemProps = Pick<
  NavListProps,
  'data' | 'depth' | 'render' | 'slotProps' | 'enabledRootRedirect'
> & {
  active: boolean;
  open: boolean;
  navItemRef: RefObject<HTMLButtonElement | null>;
  onClick: () => void;
};

function NavListItem({
  data,
  depth,
  render,
  slotProps,
  active,
  open,
  navItemRef,
  enabledRootRedirect,
  onClick,
}: NavListItemProps) {
  return (
    <NavItem
      ref={navItemRef}
      path={data.path}
      icon={data.icon}
      info={data.info}
      title={data.title}
      caption={data.caption}
      open={open}
      active={active}
      disabled={data.disabled}
      depth={depth}
      render={render}
      hasChild={!!data.children}
      externalLink={isExternalLink(data.path)}
      enabledRootRedirect={enabledRootRedirect}
      slotProps={depth === 1 ? slotProps?.rootItem : slotProps?.subItem}
      onClick={onClick}
    />
  );
}

type NavListCollapseProps = Pick<
  NavListProps,
  'data' | 'depth' | 'render' | 'slotProps' | 'checkPermissions' | 'enabledRootRedirect'
> & {
  open: boolean;
};

function NavListCollapse({
  data,
  depth,
  render,
  slotProps,
  open,
  checkPermissions,
  enabledRootRedirect,
}: NavListCollapseProps) {
  if (!data.children) {
    return null;
  }

  return (
    <NavCollapse mountOnEnter unmountOnExit depth={depth} in={open} data-group={data.title}>
      <NavSubList
        data={data.children}
        render={render}
        depth={depth}
        slotProps={slotProps}
        checkPermissions={checkPermissions}
        enabledRootRedirect={enabledRootRedirect}
      />
    </NavCollapse>
  );
}

// ----------------------------------------------------------------------

function NavSubList({
  data,
  render,
  depth = 0,
  slotProps,
  checkPermissions,
  enabledRootRedirect,
}: NavSubListProps) {
  return (
    <NavUl sx={{ gap: 'var(--nav-item-gap)' }}>
      {data.map((list, index) => (
        <NavList
          key={navItemKey(list, index)}
          data={list}
          render={render}
          depth={depth + 1}
          slotProps={slotProps}
          checkPermissions={checkPermissions}
          enabledRootRedirect={enabledRootRedirect}
        />
      ))}
    </NavUl>
  );
}
