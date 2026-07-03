import type { NavListProps } from '../types';

import { useRef, useEffect, useCallback } from 'react';
import { isActiveLink, isExternalLink } from 'minimal-shared/utils';

import Collapse from '@mui/material/Collapse';

import { usePathname } from 'src/routes/hooks';

import { megaMenuClasses } from '../styles';
import { Nav, NavUl, NavLi, NavItem, NavSubList } from '../components';

// ----------------------------------------------------------------------

export type NavListCollapseProps = NavListProps & {
  expandedPath: string | null;
  setExpandedPath: React.Dispatch<React.SetStateAction<string | null>>;
  setLastExpandedPath: React.Dispatch<React.SetStateAction<string | null>>;
};

export function NavListCollapse({
  data,
  render,
  cssVars,
  slotProps,
  /********/
  expandedPath,
  setExpandedPath,
  setLastExpandedPath,
}: NavListCollapseProps) {
  const pathname = usePathname();
  const navItemRef = useRef<HTMLButtonElement>(null);

  const isActive = isActiveLink(pathname, data.path, data.deepMatch ?? !!data.children);
  const isExpanded = expandedPath === data.path;

  useEffect(() => {
    if (isActive && data.children) {
      setExpandedPath(data.path);
      setLastExpandedPath(data.path);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname]);

  const handleToggleExpand = useCallback(() => {
    if (data.children) {
      setExpandedPath((prev) => (prev === data.path ? null : data.path));
    }
  }, [data.children, data.path, setExpandedPath]);

  const renderNavItem = () => (
    <NavItem
      ref={navItemRef}
      // slots
      path={data.path}
      icon={data.icon}
      info={data.info}
      title={data.title}
      // state
      open={isExpanded}
      active={isActive}
      disabled={data.disabled}
      // options
      render={render}
      hasChild={!!data.children}
      externalLink={isExternalLink(data.path)}
      // styles
      slotProps={slotProps?.rootItem}
      // actions
      onClick={handleToggleExpand}
    />
  );

  const renderCollapse = () =>
    !!data.children && (
      <Collapse mountOnEnter unmountOnExit in={isExpanded}>
        <Nav sx={{ ...cssVars, py: 1.5, px: 2.5 }}>
          <NavUl
            sx={{
              gap: 2,
              [`& .${megaMenuClasses.ul} > .${megaMenuClasses.li}`]: {
                '--dot-size': '4px',
                alignItems: 'center',
                '&::before': {
                  mr: 2,
                  ml: 'calc((var(--nav-icon-size) - var(--dot-size)) / 2)',
                  content: "''",
                  borderRadius: '50%',
                  bgcolor: 'text.disabled',
                  width: 'var(--dot-size)',
                  height: 'var(--dot-size)',
                },
              },
            }}
          >
            <NavSubList data={data.children} slotProps={slotProps} />
          </NavUl>
        </Nav>
      </Collapse>
    );

  return (
    <NavLi
      disabled={data.disabled}
      sx={{
        [`& .${megaMenuClasses.item.arrow}`]: {
          ...(isExpanded && { transform: 'rotate(90deg)' }),
        },
      }}
    >
      {renderNavItem()}
      {renderCollapse()}
    </NavLi>
  );
}
