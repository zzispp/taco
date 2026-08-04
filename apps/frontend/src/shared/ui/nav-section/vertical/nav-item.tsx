'use client';

import type { Theme, CSSObject } from '@mui/material/styles';
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
        <NavItemTitle
          title={title}
          caption={caption}
          slotProps={slotProps}
          ownerState={ownerState}
        />
      )}
      {info && (
        <ItemInfo {...ownerState} className={navSectionClasses.item.info} sx={slotProps?.info}>
          {navItem.renderInfo}
        </ItemInfo>
      )}
      {hasChild && (
        <ItemArrow
          {...ownerState}
          icon={ownerState.open ? 'eva:arrow-ios-downward-fill' : 'eva:arrow-ios-forward-fill'}
          className={navSectionClasses.item.arrow}
          sx={slotProps?.arrow}
        />
      )}
    </>
  );
}

function NavItemTitle({
  title,
  caption,
  slotProps,
  ownerState,
}: Pick<NavItemProps, 'title' | 'caption' | 'slotProps'> & { ownerState: StyledState }) {
  return (
    <ItemTexts {...ownerState} className={navSectionClasses.item.texts} sx={slotProps?.texts}>
      <ItemTitle {...ownerState} className={navSectionClasses.item.title} sx={slotProps?.title}>
        {title}
      </ItemTitle>
      {caption && (
        <Tooltip title={caption} placement="top-start">
          <ItemCaptionText
            {...ownerState}
            className={navSectionClasses.item.caption}
            sx={slotProps?.caption}
          >
            {caption}
          </ItemCaptionText>
        </Tooltip>
      )}
    </ItemTexts>
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

type ItemStyleState = Pick<NavItemProps, 'open' | 'active'> & { theme: Theme };

const shouldForwardProp = (prop: string) =>
  !['open', 'active', 'disabled', 'variant', 'sx'].includes(prop);

/**
 * @slot root
 */
const BULLET_SVG = `"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='14' height='14' fill='none' viewBox='0 0 14 14'%3E%3Cpath d='M1 1v4a8 8 0 0 0 8 8h4' stroke='%23efefef' stroke-width='2' stroke-linecap='round'/%3E%3C/svg%3E"`;

const ItemRoot = styled(ButtonBase, { shouldForwardProp })<StyledState>((state) =>
  createItemRootStyles(state)
);

function createItemRootStyles({ active, open, theme }: ItemStyleState): CSSObject {
  return {
    width: '100%',
    paddingTop: 'var(--nav-item-pt)',
    paddingLeft: 'var(--nav-item-pl)',
    paddingRight: 'var(--nav-item-pr)',
    paddingBottom: 'var(--nav-item-pb)',
    borderRadius: 'var(--nav-item-radius)',
    color: 'var(--nav-item-color)',
    '&:hover': { backgroundColor: 'var(--nav-item-hover-bg)' },
    variants: [
      { props: { variant: 'rootItem' }, style: createRootItemStyles({ active, open, theme }) },
      { props: { variant: 'subItem' }, style: createSubItemStyles({ active, open, theme }) },
      { props: { disabled: true }, style: navItemStyles.disabled },
    ],
  };
}

function createRootItemStyles({ active, open, theme }: ItemStyleState): CSSObject {
  return {
    minHeight: 'var(--nav-item-root-height)',
    ...(open && {
      color: 'var(--nav-item-root-open-color)',
      backgroundColor: 'var(--nav-item-root-open-bg)',
    }),
    ...(active && {
      color: 'var(--nav-item-root-active-color)',
      backgroundColor: 'var(--nav-item-root-active-bg)',
      '&:hover': { backgroundColor: 'var(--nav-item-root-active-hover-bg)' },
      ...theme.applyStyles('dark', { color: 'var(--nav-item-root-active-color-on-dark)' }),
    }),
  };
}

function createSubItemStyles({ active, open, theme }: ItemStyleState): CSSObject {
  return {
    minHeight: 'var(--nav-item-sub-height)',
    '&::before': createBulletStyles(theme),
    ...(open && {
      color: 'var(--nav-item-sub-open-color)',
      backgroundColor: 'var(--nav-item-sub-open-bg)',
    }),
    ...(active && {
      color: 'var(--nav-item-sub-active-color)',
      backgroundColor: 'var(--nav-item-sub-active-bg)',
    }),
  };
}

function createBulletStyles(theme: Theme): CSSObject {
  return {
    left: 0,
    content: '""',
    position: 'absolute',
    width: 'var(--nav-bullet-size)',
    height: 'var(--nav-bullet-size)',
    backgroundColor: 'var(--nav-bullet-light-color)',
    mask: `url(${BULLET_SVG}) no-repeat 50% 50%/100% auto`,
    WebkitMask: `url(${BULLET_SVG}) no-repeat 50% 50%/100% auto`,
    transform:
      theme.direction === 'rtl'
        ? 'translate(calc(var(--nav-bullet-size) * 1), calc(var(--nav-bullet-size) * -0.4)) scaleX(-1)'
        : 'translate(calc(var(--nav-bullet-size) * -1), calc(var(--nav-bullet-size) * -0.4))',
    ...theme.applyStyles('dark', { backgroundColor: 'var(--nav-bullet-dark-color)' }),
  };
}

/**
 * @slot icon
 */
const ItemIcon = styled('span', { shouldForwardProp })<StyledState>(() => ({
  ...navItemStyles.icon,
  width: 'var(--nav-icon-size)',
  height: 'var(--nav-icon-size)',
  margin: 'var(--nav-icon-margin)',
}));

/**
 * @slot texts
 */
const ItemTexts = styled('span', { shouldForwardProp })<StyledState>(() => ({
  ...navItemStyles.texts,
}));

/**
 * @slot title
 */
const ItemTitle = styled('span', { shouldForwardProp })<StyledState>(({ theme }) => ({
  ...navItemStyles.title(theme),
  ...theme.typography.body2,
  fontWeight: theme.typography.fontWeightMedium,
  variants: [
    { props: { active: true }, style: { fontWeight: theme.typography.fontWeightSemiBold } },
  ],
}));

/**
 * @slot caption text
 */
const ItemCaptionText = styled('span', { shouldForwardProp })<StyledState>(({ theme }) => ({
  ...navItemStyles.captionText(theme),
  color: 'var(--nav-item-caption-color)',
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
}));
