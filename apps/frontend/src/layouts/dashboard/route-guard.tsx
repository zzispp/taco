'use client';

import type { NavSectionProps } from 'src/components/nav-section';

import { m } from 'framer-motion';

import Alert from '@mui/material/Alert';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { paths } from 'src/routes/paths';
import { usePathname } from 'src/routes/hooks';

import { ForbiddenIllustration } from 'src/assets/illustrations';

import { LoadingScreen } from 'src/components/loading-screen';
import { varBounce, MotionContainer } from 'src/components/animate';

// ----------------------------------------------------------------------

const PATH_SUFFIX_PATTERN = /[?#]/;
const TRAILING_SLASH_PATTERN = /\/+$/;

type NavData = NavSectionProps['data'];
type NavItem = NavData[number]['items'][number];

type DashboardRouteGuardProps = {
  children: React.ReactNode;
  data: NavData;
  error?: unknown;
  isLoading: boolean;
};

export function DashboardRouteGuard({
  data,
  error,
  children,
  isLoading,
}: DashboardRouteGuardProps) {
  const pathname = usePathname();

  if (!isDashboardPath(pathname)) {
    return <>{children}</>;
  }

  if (isLoading) {
    return <LoadingScreen portal={false} />;
  }

  if (error) {
    return <DashboardRouteError error={error} />;
  }

  if (!hasDashboardRoute(data, pathname)) {
    return <DashboardPermissionDenied />;
  }

  return <>{children}</>;
}

function hasDashboardRoute(data: NavData, pathname: string) {
  const route = normalizePath(pathname);
  return data.some((section) => section.items.some((item) => itemMatchesRoute(item, route)));
}

function itemMatchesRoute(item: NavItem, route: string): boolean {
  const itemPath = normalizePath(item.path);

  if (!isDashboardPath(itemPath)) {
    return false;
  }

  if (route === itemPath) {
    return true;
  }

  if (allowsSubRoutes(item) && route.startsWith(`${itemPath}/`)) {
    return true;
  }

  return item.children?.some((child) => itemMatchesRoute(child, route)) ?? false;
}

function allowsSubRoutes(item: NavItem) {
  return item.deepMatch ?? Boolean(item.children?.length);
}

function isDashboardPath(pathname: string) {
  const route = normalizePath(pathname);
  return route === paths.dashboard.root || route.startsWith(`${paths.dashboard.root}/`);
}

function normalizePath(path: string) {
  const [pathname] = path.split(PATH_SUFFIX_PATTERN);
  const normalized = pathname.replace(TRAILING_SLASH_PATTERN, '');
  return normalized || '/';
}

function DashboardRouteError({ error }: { error: unknown }) {
  return (
    <Container sx={{ py: 10 }}>
      <Alert severity="error">{errorMessage(error)}</Alert>
    </Container>
  );
}

function DashboardPermissionDenied() {
  return (
    <Container component={MotionContainer} sx={{ py: 10, textAlign: 'center' }}>
      <m.div variants={varBounce('in')}>
        <Typography variant="h3" sx={{ mb: 2 }}>
          Permission denied
        </Typography>
      </m.div>

      <m.div variants={varBounce('in')}>
        <Typography sx={{ color: 'text.secondary' }}>
          You do not have permission to access this page.
        </Typography>
      </m.div>

      <m.div variants={varBounce('in')}>
        <ForbiddenIllustration sx={{ my: { xs: 5, sm: 10 } }} />
      </m.div>
    </Container>
  );
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : 'Failed to load dashboard routes.';
}
