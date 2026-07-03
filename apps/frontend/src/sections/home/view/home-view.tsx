'use client';

import type { BoxProps } from '@mui/material/Box';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import { alpha } from '@mui/material/styles';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { paths } from 'src/routes/paths';
import { RouterLink } from 'src/routes/components';

import { BackToTopButton } from 'src/components/animate/back-to-top-button';
import { ScrollProgress, useScrollProgress } from 'src/components/animate/scroll-progress';

import { HeroBackground } from '../components/hero-background';

const CAPABILITIES = ['Authentication', 'RBAC', 'APIs', 'Menus'];

const PRINCIPLES = [
  {
    title: 'Identity and access',
    description: 'Centralize local authentication, roles, and access control in one backend console.',
  },
  {
    title: 'Explicit permissions',
    description: 'Operate API permissions and menu visibility with one RBAC source of truth.',
  },
  {
    title: 'Operational simplicity',
    description: 'Keep system users, menus, and admin capabilities aligned without template drift.',
  },
];

const ENTRY_POINTS = [
  {
    title: 'User management',
    description: 'Open the admin workspace at the default users console.',
    href: paths.dashboard.admin.users,
  },
  {
    title: 'Role and API control',
    description: 'Manage roles, API permissions, and menus from the same backend surface.',
    href: paths.dashboard.admin.roles,
  },
  {
    title: 'Authentication',
    description: 'Use the current sign-in and sign-up flow for the Hook control plane.',
    href: paths.auth.jwt.signIn,
  },
];

export function HomeView() {
  const pageProgress = useScrollProgress();

  return (
    <>
      <ScrollProgress
        variant="linear"
        progress={pageProgress.scrollYProgress}
        sx={[(theme) => ({ position: 'fixed', zIndex: theme.zIndex.appBar + 1 })]}
      />

      <BackToTopButton />

      <HeroSection />

      <Stack sx={{ position: 'relative', bgcolor: 'background.default' }}>
        <SectionShell
          title="What remains after template cleanup"
          description="Hook now keeps only the surfaces that serve the backend control plane: auth, users, RBAC, APIs, and menus."
        >
          <Grid container spacing={3}>
            {PRINCIPLES.map((item) => (
              <Grid key={item.title} size={{ xs: 12, md: 4 }}>
                <SectionCard title={item.title} description={item.description} />
              </Grid>
            ))}
          </Grid>
        </SectionShell>

        <SectionShell
          title="Primary entry points"
          description="The homepage now routes directly into the same modules that the backend and the current UI actively use."
        >
          <Grid container spacing={3}>
            {ENTRY_POINTS.map((item) => (
              <Grid key={item.title} size={{ xs: 12, md: 4 }}>
                <SectionCard
                  title={item.title}
                  description={item.description}
                  action={
                    <Button component={RouterLink} href={item.href} variant="text" color="inherit">
                      Open
                    </Button>
                  }
                />
              </Grid>
            ))}
          </Grid>
        </SectionShell>
      </Stack>
    </>
  );
}

function HeroSection() {
  return (
    <Box
      component="section"
      sx={(theme) => ({
        overflow: 'hidden',
        position: 'relative',
        pt: { xs: 14, md: 20 },
        pb: { xs: 10, md: 16 },
        mt: { md: 'calc(var(--layout-header-desktop-height) * -1)' },
      })}
    >
      <HeroBackground />

      <Container sx={{ position: 'relative' }}>
        <Stack spacing={4} alignItems="center" sx={{ textAlign: 'center', maxWidth: 840, mx: 'auto' }}>
          <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap" justifyContent="center">
            {CAPABILITIES.map((item) => (
              <Chip key={item} label={item} color="default" variant="outlined" />
            ))}
          </Stack>

          <Typography variant="h1" sx={{ typography: { xs: 'h2', md: 'h1' }, maxWidth: 760 }}>
            Hook runs your backend control plane from one console.
          </Typography>

          <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 640, fontWeight: 400 }}>
            Centralize authentication, RBAC, API permissions, and menu governance with one
            operational surface.
          </Typography>

          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
            <Button component={RouterLink} href={paths.dashboard.root} size="large" variant="contained">
              Open console
            </Button>
            <Button component={RouterLink} href={paths.auth.jwt.signIn} size="large" variant="outlined">
              Sign in
            </Button>
          </Stack>
        </Stack>
      </Container>
    </Box>
  );
}

function SectionShell({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <Box component="section" sx={{ py: { xs: 8, md: 12 } }}>
      <Container>
        <Stack spacing={2} sx={{ mb: 5, maxWidth: 720 }}>
          <Typography variant="overline" sx={{ color: 'text.disabled' }}>
            Hook
          </Typography>
          <Typography variant="h3">{title}</Typography>
          <Typography color="text.secondary">{description}</Typography>
        </Stack>
        {children}
      </Container>
    </Box>
  );
}

function SectionCard({
  title,
  description,
  action,
  sx,
}: BoxProps & {
  title: string;
  description: string;
  action?: React.ReactNode;
}) {
  return (
    <Box
      sx={[
        (theme) => ({
          height: 1,
          p: 3,
          borderRadius: 3,
          border: `1px solid ${alpha(theme.palette.grey[500], 0.16)}`,
          background: `linear-gradient(180deg, ${theme.vars.palette.background.paper} 0%, ${alpha(theme.palette.primary.main, 0.03)} 100%)`,
        }),
        ...(Array.isArray(sx) ? sx : [sx]),
      ]}
    >
      <Stack spacing={2}>
        <Typography variant="h5">{title}</Typography>
        <Typography color="text.secondary">{description}</Typography>
        {action}
      </Stack>
    </Box>
  );
}
