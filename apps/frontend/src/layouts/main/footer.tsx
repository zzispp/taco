import type { Theme, Breakpoint } from '@mui/material/styles';

import Box from '@mui/material/Box';
import Link from '@mui/material/Link';
import Grid from '@mui/material/Grid';
import Divider from '@mui/material/Divider';
import { styled } from '@mui/material/styles';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { Logo } from 'src/components/logo';

// ----------------------------------------------------------------------

const LINKS = [
  {
    headline: 'Product',
    children: [
      { name: 'Home', href: paths.home },
      { name: 'Console', href: paths.dashboard.root },
      { name: 'User management', href: paths.dashboard.admin.users },
    ],
  },
  {
    headline: 'Access',
    children: [
      { name: 'Sign in', href: paths.auth.jwt.signIn },
      { name: 'Sign up', href: paths.auth.jwt.signUp },
    ],
  },
];

const FooterRoot = styled('footer')(({ theme }) => ({
  position: 'relative',
  backgroundColor: theme.vars.palette.background.default,
}));

export type FooterProps = React.ComponentProps<typeof FooterRoot>;

export function Footer({
  sx,
  layoutQuery = 'md',
  ...other
}: FooterProps & { layoutQuery?: Breakpoint }) {
  return (
    <FooterRoot sx={sx} {...other}>
      <Divider />
      <Container sx={footerContainerSx(layoutQuery)}>
        <Logo />
        <Grid container sx={footerGridSx(layoutQuery)}>
          <Grid size={{ xs: 12, [layoutQuery]: 4 }}>
            <Typography variant="body2" sx={footerTextSx(layoutQuery)}>
              Hook is the management console for authentication, RBAC, API permissions, and menu
              governance in one backend control plane.
            </Typography>
          </Grid>
          <Grid size={{ xs: 12, [layoutQuery]: 5 }}>
            <Grid container spacing={3}>
              {LINKS.map((group) => (
                <Grid key={group.headline} size={{ xs: 12, sm: 6 }}>
                  <FooterColumn headline={group.headline} links={group.children} layoutQuery={layoutQuery} />
                </Grid>
              ))}
            </Grid>
          </Grid>
        </Grid>
        <Typography variant="body2" sx={{ mt: 10 }}>
          © Hook. All rights reserved.
        </Typography>
      </Container>
    </FooterRoot>
  );
}

export function HomeFooter({ sx, ...other }: FooterProps) {
  return (
    <FooterRoot sx={[{ py: 5, textAlign: 'center' }, ...(Array.isArray(sx) ? sx : [sx])]} {...other}>
      <Container>
        <Logo />
        <Box sx={{ mt: 1, typography: 'caption' }}>Hook backend control plane.</Box>
      </Container>
    </FooterRoot>
  );
}

function FooterColumn({
  headline,
  links,
  layoutQuery,
}: {
  headline: string;
  links: { name: string; href: string }[];
  layoutQuery: Breakpoint;
}) {
  return (
    <Box sx={footerColumnSx(layoutQuery)}>
      <Typography component="div" variant="overline">
        {headline}
      </Typography>
      {links.map((link) => (
        <Link key={link.name} component={RouterLink} href={link.href} color="inherit" variant="body2">
          {link.name}
        </Link>
      ))}
    </Box>
  );
}

function footerContainerSx(layoutQuery: Breakpoint) {
  return (theme: Theme) => ({
    pb: 5,
    pt: 10,
    textAlign: 'center',
    [theme.breakpoints.up(layoutQuery)]: { textAlign: 'unset' },
  });
}

function footerGridSx(layoutQuery: Breakpoint) {
  return (theme: Theme) => ({
    mt: 3,
    justifyContent: 'center',
    [theme.breakpoints.up(layoutQuery)]: { justifyContent: 'space-between' },
  });
}

function footerTextSx(layoutQuery: Breakpoint) {
  return (theme: Theme) => ({
    mx: 'auto',
    maxWidth: 320,
    [theme.breakpoints.up(layoutQuery)]: { mx: 'unset' },
  });
}

function footerColumnSx(layoutQuery: Breakpoint) {
  return (theme: Theme) => ({
    gap: 2,
    width: 1,
    display: 'flex',
    alignItems: 'center',
    flexDirection: 'column',
    [theme.breakpoints.up(layoutQuery)]: { alignItems: 'flex-start' },
  });
}
