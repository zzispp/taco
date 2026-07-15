import type { BoxProps } from '@mui/material/Box';
import type { Theme, Breakpoint } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import { useTheme } from '@mui/material/styles';
import IconButton from '@mui/material/IconButton';
import useMediaQuery from '@mui/material/useMediaQuery';

import { Label } from 'src/shared/ui/label';
import { Iconify } from 'src/shared/ui/iconify';

const BREAKPOINT: Breakpoint = 'sm';

type SearchButtonProps = BoxProps & {
  onOpen: () => void;
};

export function SearchButton({ onOpen, sx, ...other }: SearchButtonProps) {
  const theme = useTheme();
  const isDesktop = useMediaQuery(theme.breakpoints.up(BREAKPOINT));

  return (
    <Box onClick={onOpen} sx={[triggerStyle(theme), ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <SearchIcon desktop={isDesktop} />
      <SearchShortcut />
    </Box>
  );
}

function SearchIcon({ desktop }: { desktop: boolean }) {
  return (
    <Box
      component={desktop ? 'span' : IconButton}
      sx={{
        [useTheme().breakpoints.up(BREAKPOINT)]: {
          p: 1,
          display: 'inline-flex',
          color: 'action.active',
        },
      }}
    >
      <Iconify icon="eva:search-fill" />
    </Box>
  );
}

function SearchShortcut() {
  return (
    <Label
      sx={(theme) => ({
        color: 'grey.800',
        cursor: 'inherit',
        bgcolor: 'common.white',
        fontSize: theme.typography.pxToRem(12),
        boxShadow: theme.vars.customShadows.z1,
        display: { xs: 'none', [BREAKPOINT]: 'inline-flex' },
      })}
    >
      ⌘K
    </Label>
  );
}

function triggerStyle(theme: Theme) {
  return {
    display: 'flex',
    alignItems: 'center',
    [theme.breakpoints.up(BREAKPOINT)]: {
      pr: 1,
      borderRadius: 1.5,
      cursor: 'pointer',
      bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.08),
      transition: theme.transitions.create('background-color', {
        easing: theme.transitions.easing.easeInOut,
        duration: theme.transitions.duration.shortest,
      }),
      '&:hover': { bgcolor: varAlpha(theme.vars.palette.grey['500Channel'], 0.16) },
    },
  };
}
