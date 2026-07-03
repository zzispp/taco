import type { NavMainProps } from '../types';

import { useEffect } from 'react';

import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import Drawer from '@mui/material/Drawer';

import { Logo } from 'src/shared/ui/logo';
import { paths } from 'src/shared/routes/paths';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { usePathname } from 'src/shared/routes/hooks';
import { RouterLink } from 'src/shared/routes/components';

import { SignInButton } from 'src/features/auth/sign-in';

import { Nav, NavUl } from '../components';
import { NavList } from './nav-mobile-list';

// ----------------------------------------------------------------------

export type NavMobileProps = NavMainProps & {
  open: boolean;
  onClose: () => void;
  slots?: {
    topArea?: React.ReactNode;
    bottomArea?: React.ReactNode;
  };
};

export function NavMobile({ data, open, onClose, slots, sx }: NavMobileProps) {
  const pathname = usePathname();

  useEffect(() => {
    if (open) {
      onClose();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [pathname]);

  return (
    <Drawer
      open={open}
      onClose={onClose}
      slotProps={{
        paper: {
          sx: [
            {
              display: 'flex',
              flexDirection: 'column',
              width: 'var(--layout-nav-mobile-width)',
            },
            ...(Array.isArray(sx) ? sx : [sx]),
          ],
        },
      }}
    >
      {slots?.topArea ?? (
        <Box
          sx={{
            pt: 3,
            pb: 2,
            pl: 2.5,
            display: 'flex',
          }}
        >
          <Logo />
        </Box>
      )}

      <Scrollbar fillContent>
        <Nav
          sx={{
            pb: 3,
            display: 'flex',
            flex: '1 1 auto',
            flexDirection: 'column',
          }}
        >
          <NavUl>
            {data.map((list) => (
              <NavList key={list.title} data={list} />
            ))}
          </NavUl>
        </Nav>
      </Scrollbar>

      {slots?.bottomArea ?? (
        <Box
          sx={{
            py: 3,
            px: 2.5,
            gap: 1.5,
            display: 'flex',
          }}
        >
          <SignInButton fullWidth />

          <Button
            fullWidth
            component={RouterLink}
            variant="contained"
            href={paths.dashboard.root}
          >
            Open console
          </Button>
        </Box>
      )}
    </Drawer>
  );
}
