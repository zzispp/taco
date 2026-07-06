'use client';

import type { IconButtonProps } from '@mui/material/IconButton';

import { useBoolean } from 'minimal-shared/hooks';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Avatar from '@mui/material/Avatar';
import Drawer from '@mui/material/Drawer';
import MenuList from '@mui/material/MenuList';
import MenuItem from '@mui/material/MenuItem';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';

import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { AnimateBorder } from 'src/shared/ui/animate';
import { RouterLink } from 'src/shared/routes/components';

import { useSessionUser } from 'src/entities/session';

import { SignOutButton } from 'src/features/auth/sign-out';

import { AccountButton } from './account-button';

// ----------------------------------------------------------------------

export type AccountDrawerProps = IconButtonProps & {
  data?: AccountLink[];
  navTitle?: string;
};

export type AccountLink = {
  key: string;
  label: string;
  href: string;
  depth?: number;
  icon?: React.ReactNode;
};

export function AccountDrawer({ data = [], navTitle, sx, ...other }: AccountDrawerProps) {
  const { user } = useSessionUser();
  const { value: open, onFalse: onClose, onTrue: onOpen } = useBoolean();

  return (
    <>
      <AccountButton
        onClick={onOpen}
        photoURL={user?.photoURL}
        displayName={user?.displayName}
        sx={sx}
        {...other}
      />

      <Drawer
        open={open}
        onClose={onClose}
        anchor="right"
        slotProps={{
          backdrop: { invisible: true },
          paper: { sx: { width: 320 } },
        }}
      >
        <IconButton onClick={onClose} sx={{ top: 12, left: 12, zIndex: 9, position: 'absolute' }}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>

        <Scrollbar>
          <AccountIdentity
            displayName={user?.displayName}
            email={user?.email}
            photoURL={user?.photoURL}
          />
          <AccountLinks data={data} title={navTitle} onClose={onClose} />
        </Scrollbar>

        <Box sx={{ p: 2.5 }}>
          <SignOutButton onClose={onClose} />
        </Box>
      </Drawer>
    </>
  );
}

function AccountIdentity({
  displayName,
  email,
  photoURL,
}: {
  displayName?: string;
  email?: string;
  photoURL?: string;
}) {
  return (
    <Box sx={{ pt: 8, display: 'flex', alignItems: 'center', flexDirection: 'column' }}>
      <AnimateBorder
        sx={{ mb: 2, p: '6px', width: 96, height: 96, borderRadius: '50%' }}
        slotProps={{ primaryBorder: { size: 120, sx: { color: 'primary.main' } } }}
      >
        <Avatar src={photoURL} alt={displayName} sx={{ width: 1, height: 1 }}>
          {displayName?.charAt(0).toUpperCase()}
        </Avatar>
      </AnimateBorder>

      <Typography variant="subtitle1" noWrap sx={{ mt: 2 }}>
        {displayName}
      </Typography>

      <Typography variant="body2" sx={{ color: 'text.secondary', mt: 0.5 }} noWrap>
        {email}
      </Typography>
    </Box>
  );
}

function AccountLinks({
  data,
  title,
  onClose,
}: {
  data: NonNullable<AccountDrawerProps['data']>;
  title?: string;
  onClose: () => void;
}) {
  return (
    <Box
      sx={[
        (theme) => ({
          py: 3,
          px: 2.5,
          borderTop: `dashed 1px ${theme.vars.palette.divider}`,
          borderBottom: `dashed 1px ${theme.vars.palette.divider}`,
          '& li': { p: 0 },
        }),
      ]}
    >
      {title ? (
        <Typography variant="overline" sx={{ px: 1, color: 'text.disabled' }}>
          {title}
        </Typography>
      ) : null}
      <MenuList disablePadding>
        {data.map((option) => (
          <MenuItem key={option.key}>
            <Link
              component={RouterLink}
              href={option.href}
              color="inherit"
              underline="none"
              onClick={onClose}
              sx={{
                p: 1,
                width: 1,
                display: 'flex',
                typography: 'body2',
                alignItems: 'center',
                color: 'text.secondary',
                '& svg': { width: 24, height: 24 },
                '&:hover': { color: 'text.primary' },
              }}
            >
              {option.icon}
              <Box component="span" sx={{ ml: 2 + (option.depth ?? 0) * 1.5 }}>
                {option.label}
              </Box>
            </Link>
          </MenuItem>
        ))}
      </MenuList>
    </Box>
  );
}
