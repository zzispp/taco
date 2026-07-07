'use client';

import type { BoxProps } from '@mui/material/Box';

import { useMemo } from 'react';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Grid from '@mui/material/Grid';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import { alpha } from '@mui/material/styles';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { paths } from 'src/shared/routes/paths';
import { RouterLink } from 'src/shared/routes/components';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { BackToTopButton } from 'src/shared/ui/animate/back-to-top-button';
import { formatHomeDocumentTitle } from 'src/shared/i18n/document-title-format';
import { ScrollProgress, useScrollProgress } from 'src/shared/ui/animate/scroll-progress';

import { HeroBackground } from 'src/widgets/home-hero';

const CAPABILITIES = ['Authentication', 'RBAC', 'APIs', 'Menus'];

const PRINCIPLES = [
  {
    title: 'Identity and access',
    description:
      'Centralize local authentication, roles, and access control in one backend console.',
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

export function HomeView() {
  const pageProgress = useScrollProgress();
  const { siteName } = useSiteDisplay();
  const entryPoints = useMemo(() => createEntryPoints(siteName), [siteName]);

  return (
    <>
      <SiteDocumentTitle title={formatHomeDocumentTitle(siteName)} />

      <ScrollProgress
        variant="linear"
        progress={pageProgress.scrollYProgress}
        sx={[(theme) => ({ position: 'fixed', zIndex: theme.zIndex.appBar + 1 })]}
      />

      <BackToTopButton />

      <HeroSection siteName={siteName} />

      <HomeSections siteName={siteName} entryPoints={entryPoints} />
    </>
  );
}

function HomeSections({ siteName, entryPoints }: { siteName: string; entryPoints: ReturnType<typeof createEntryPoints> }) {
  return (
    <Stack sx={{ position: 'relative', bgcolor: 'background.default' }}>
      <PrinciplesSection siteName={siteName} />
      <EntryPointsSection siteName={siteName} entryPoints={entryPoints} />
    </Stack>
  );
}

function PrinciplesSection({ siteName }: { siteName: string }) {
  return (
    <SectionShell
      overline={siteName}
      title="What remains after template cleanup"
      description={`${siteName} now keeps only the surfaces that serve the backend control plane: auth, users, RBAC, APIs, and menus.`}
    >
      <Grid container spacing={3}>
        {PRINCIPLES.map((item) => (
          <Grid key={item.title} size={{ xs: 12, md: 4 }}>
            <SectionCard title={item.title} description={item.description} />
          </Grid>
        ))}
      </Grid>
    </SectionShell>
  );
}

function EntryPointsSection(props: { siteName: string; entryPoints: ReturnType<typeof createEntryPoints> }) {
  return (
    <SectionShell
      overline={props.siteName}
      title="Primary entry points"
      description="The homepage now routes directly into the same modules that the backend and the current UI actively use."
    >
      <Grid container spacing={3}>
        {props.entryPoints.map((item) => (
          <Grid key={item.title} size={{ xs: 12, md: 4 }}>
            <SectionCard title={item.title} description={item.description} action={<OpenEntryPointButton href={item.href} />} />
          </Grid>
        ))}
      </Grid>
    </SectionShell>
  );
}

function OpenEntryPointButton({ href }: { href: string }) {
  return (
    <Button component={RouterLink} href={href} variant="text" color="inherit">
      Open
    </Button>
  );
}

function createEntryPoints(siteName: string) {
  return [
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
      description: `Use the current sign-in and sign-up flow for the ${siteName} control plane.`,
      href: paths.auth.jwt.signIn,
    },
  ];
}

function HeroSection({ siteName }: { siteName: string }) {
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
      <HeroBackground text={`${siteName} Gateway Console`} />

      <Container sx={{ position: 'relative' }}>
        <Stack
          spacing={4}
          alignItems="center"
          sx={{ textAlign: 'center', maxWidth: 840, mx: 'auto' }}
        >
          <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap" justifyContent="center">
            {CAPABILITIES.map((item) => (
              <Chip key={item} label={item} color="default" variant="outlined" />
            ))}
          </Stack>

          <Typography variant="h1" sx={{ typography: { xs: 'h2', md: 'h1' }, maxWidth: 760 }}>
            {siteName} runs your backend control plane from one console.
          </Typography>

          <Typography variant="h6" color="text.secondary" sx={{ maxWidth: 640, fontWeight: 400 }}>
            Centralize authentication, RBAC, API permissions, and menu governance with one
            operational surface.
          </Typography>

          <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
            <Button
              component={RouterLink}
              href={paths.dashboard.root}
              size="large"
              variant="contained"
            >
              Open console
            </Button>
            <Button
              component={RouterLink}
              href={paths.auth.jwt.signIn}
              size="large"
              variant="outlined"
            >
              Sign in
            </Button>
          </Stack>
        </Stack>
      </Container>
    </Box>
  );
}

function SectionShell({
  overline,
  title,
  description,
  children,
}: {
  overline: string;
  title: string;
  description: string;
  children: React.ReactNode;
}) {
  return (
    <Box component="section" sx={{ py: { xs: 8, md: 12 } }}>
      <Container>
        <Stack spacing={2} sx={{ mb: 5, maxWidth: 720 }}>
          <Typography variant="overline" sx={{ color: 'text.disabled' }}>
            {overline}
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
