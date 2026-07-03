import type { NavListProps } from '../types';

import { useRef, useCallback } from 'react';
import { useBoolean } from 'minimal-shared/hooks';
import { isActiveLink, isExternalLink } from 'minimal-shared/utils';

import Collapse from '@mui/material/Collapse';

import { usePathname } from 'src/shared/routes/hooks';
import { NavSectionVertical } from 'src/shared/ui/nav-section';

import { NavLi } from '../components';
import { NavItem } from './nav-mobile-item';

// ----------------------------------------------------------------------

export function NavList({ data, sx, ...other }: NavListProps) {
  const pathname = usePathname();
  const navItemRef = useRef<HTMLButtonElement>(null);
  const isActive = isActiveLink(pathname, data.path, data.deepMatch ?? !!data.children);
  const isOpenPath = !!data.children && isActive;

  const { value: open, onToggle } = useBoolean(isOpenPath);

  const handleToggleMenu = useCallback(() => {
    if (data.children) {
      onToggle();
    }
  }, [data.children, onToggle]);

  const renderNavItem = () => (
    <NavItem
      ref={navItemRef}
      // slots
      path={data.path}
      icon={data.icon}
      title={data.title}
      // state
      open={open}
      active={isActive}
      // options
      hasChild={!!data.children}
      externalLink={isExternalLink(data.path)}
      // actions
      onClick={handleToggleMenu}
    />
  );

  const renderCollapse = () =>
    !!data.children && (
      <Collapse in={open}>
        <NavSectionVertical data={data.children} sx={{ px: 1.5 }} />
      </Collapse>
    );

  return (
    <NavLi sx={sx} {...other}>
      {renderNavItem()}
      {renderCollapse()}
    </NavLi>
  );
}
