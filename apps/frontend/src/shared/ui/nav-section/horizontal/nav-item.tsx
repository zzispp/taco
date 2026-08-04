'use client';

import type { CSSObject } from '@mui/material/styles';
import type { NavItemProps } from '../types';

import { mergeClasses } from 'minimal-shared/utils';

import Tooltip from '@mui/material/Tooltip';
import { styled } from '@mui/material/styles';
import ButtonBase from '@mui/material/ButtonBase';

import { Iconify } from '../../iconify';
import { createNavItem } from '../utils';
import { navItemStyles, navSectionClasses } from '../styles';

// ----------------------------------------------------------------------

export function NavItem(props: NavItemProps) {
  const navItem = createNavItem(props);
  const ownerState = createOwnerState(props, navItem);

  return <NavItemRoot {...props} navItem={navItem} ownerState={ownerState} />;
}

type NavItemRootProps = NavItemProps & {
  navItem: ReturnType<typeof createNavItem>;
  ownerState: StyledState;
};

function NavItemRoot({
  path: _path,
  icon,
  info,
  title,
  caption,
  open: _open,
  active: _active,
  disabled: _disabled,
  depth: _depth,
  render: _render,
  hasChild,
  slotProps,
  className,
  externalLink: _externalLink,
  enabledRootRedirect: _enabledRootRedirect,
  navItem,
  ownerState,
  ...other
}: NavItemRootProps) {
  return (
    <ItemRoot
      aria-label={title}
      {...ownerState}
      {...navItem.baseProps}
      className={mergeClasses([navSectionClasses.item.root, className], {
        [navSectionClasses.state.open]: ownerState.open,
        [navSectionClasses.state.active]: ownerState.active,
        [navSectionClasses.state.disabled]: ownerState.disabled,
      })}
      sx={slotProps?.sx}
      {...other}
    >
      <NavItemContent
        icon={icon}
        info={info}
        title={title}
        caption={caption}
        hasChild={hasChild}
        navItem={navItem}
        ownerState={ownerState}
        slotProps={slotProps}
      />
    </ItemRoot>
  );
}

// ----------------------------------------------------------------------

type NavItemContentProps = Pick<
  NavItemProps,
  'icon' | 'info' | 'title' | 'caption' | 'hasChild' | 'slotProps'
> & {
  navItem: ReturnType<typeof createNavItem>;
  ownerState: StyledState;
};

function NavItemContent({
  icon,
  info,
  title,
  caption,
  hasChild,
  slotProps,
  navItem,
  ownerState,
}: NavItemContentProps) {
  return (
    <>
      {icon && (
        <ItemIcon {...ownerState} className={navSectionClasses.item.icon} sx={slotProps?.icon}>
          {navItem.renderIcon}
        </ItemIcon>
      )}
      {title && (
        <ItemTitle {...ownerState} className={navSectionClasses.item.title} sx={slotProps?.title}>
          {title}
        </ItemTitle>
      )}
      {caption && (
        <Tooltip title={caption} arrow>
          <ItemCaptionIcon
            {...ownerState}
            icon="eva:info-outline"
            className={navSectionClasses.item.caption}
            sx={slotProps?.caption}
          />
        </Tooltip>
      )}
      {info && (
        <ItemInfo {...ownerState} className={navSectionClasses.item.info} sx={slotProps?.info}>
          {navItem.renderInfo}
        </ItemInfo>
      )}
      {hasChild && (
        <ItemArrow
          {...ownerState}
          icon={navItem.subItem ? 'eva:arrow-ios-forward-fill' : 'eva:arrow-ios-downward-fill'}
          className={navSectionClasses.item.arrow}
          sx={slotProps?.arrow}
        />
      )}
    </>
  );
}

function createOwnerState(
  { open, active, disabled }: NavItemProps,
  navItem: ReturnType<typeof createNavItem>
): StyledState {
  return { open, active, disabled, variant: navItem.rootItem ? 'rootItem' : 'subItem' };
}

// ----------------------------------------------------------------------

type StyledState = Pick<NavItemProps, 'open' | 'active' | 'disabled'> & {
  variant: 'rootItem' | 'subItem';
};

const shouldForwardProp = (prop: string) =>
  !['open', 'active', 'disabled', 'variant', 'sx'].includes(prop);

/**
 * @slot root
 */
const ItemRoot = styled(ButtonBase, { shouldForwardProp })<StyledState>(({
  active,
  open,
  theme,
}) => {
  const rootItemStyles: CSSObject = {
    padding: 'var(--nav-item-root-padding)',
    minHeight: 'var(--nav-item-root-height)',
    ...(open && {
      color: 'var(--nav-item-root-open-color)',
      backgroundColor: 'var(--nav-item-root-open-bg)',
    }),
    ...(active && {
      color: 'var(--nav-item-root-active-color)',
      backgroundColor: 'var(--nav-item-root-active-bg)',
      '&:hover': { backgroundColor: 'var(--nav-item-root-active-hover-bg)' },
      ...theme.applyStyles('dark', {
        color: 'var(--nav-item-root-active-color-on-dark)',
      }),
    }),
  };

  const subItemStyles: CSSObject = {
    padding: 'var(--nav-item-sub-padding)',
    minHeight: 'var(--nav-item-sub-height)',
    color: theme.vars.palette.text.secondary,
    ...(open && {
      color: 'var(--nav-item-sub-open-color)',
      backgroundColor: 'var(--nav-item-sub-open-bg)',
    }),
    ...(active && {
      color: 'var(--nav-item-sub-active-color)',
      backgroundColor: 'var(--nav-item-sub-active-bg)',
    }),
  };

  return {
    width: '100%',
    flexShrink: 0,
    color: 'var(--nav-item-color)',
    borderRadius: 'var(--nav-item-radius)',
    '&:hover': { backgroundColor: 'var(--nav-item-hover-bg)' },
    variants: [
      { props: { variant: 'rootItem' }, style: rootItemStyles },
      { props: { variant: 'subItem' }, style: subItemStyles },
      { props: { disabled: true }, style: navItemStyles.disabled },
    ],
  };
});

/**
 * @slot icon
 */
const ItemIcon = styled('span', { shouldForwardProp })<StyledState>(() => ({
  ...navItemStyles.icon,
  width: 'var(--nav-icon-size)',
  height: 'var(--nav-icon-size)',
  margin: 'var(--nav-icon-root-margin)',
  variants: [{ props: { variant: 'subItem' }, style: { margin: 'var(--nav-icon-sub-margin)' } }],
}));

/**
 * @slot title
 */
const ItemTitle = styled('span', { shouldForwardProp })<StyledState>(({ theme }) => ({
  ...navItemStyles.title(theme),
  ...theme.typography.body2,
  whiteSpace: 'nowrap',
  fontWeight: theme.typography.fontWeightMedium,
  variants: [
    { props: { active: true }, style: { fontWeight: theme.typography.fontWeightSemiBold } },
  ],
}));

/**
 * @slot caption icon
 */
const ItemCaptionIcon = styled(Iconify, { shouldForwardProp })<StyledState>(({ theme }) => ({
  ...navItemStyles.captionIcon,
  color: 'var(--nav-item-caption-color)',
  variants: [{ props: { variant: 'rootItem' }, style: { marginLeft: theme.spacing(0.75) } }],
}));

/**
 * @slot info
 */
const ItemInfo = styled('span', { shouldForwardProp })<StyledState>(({ theme }) => ({
  ...navItemStyles.info,
}));

/**
 * @slot arrow
 */
const ItemArrow = styled(Iconify, { shouldForwardProp })<StyledState>(({ theme }) => ({
  ...navItemStyles.arrow(theme),
  variants: [{ props: { variant: 'subItem' }, style: { marginRight: theme.spacing(-0.5) } }],
}));
