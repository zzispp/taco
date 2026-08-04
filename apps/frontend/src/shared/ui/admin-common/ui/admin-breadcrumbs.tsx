'use client';

import Stack from '@mui/material/Stack';

import { paths } from 'src/shared/routes/paths';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { CustomBreadcrumbs } from 'src/shared/ui/custom-breadcrumbs';

import { PageRefreshButton } from './page-refresh-button';

type AdminBreadcrumbsProps = {
  heading: string;
  action?: React.ReactNode;
  onRefresh?: () => Promise<void> | void;
  refreshing?: boolean;
  parentLinks?: readonly AdminBreadcrumbLink[];
};

type AdminBreadcrumbLink = Readonly<{ name: string; href?: string }>;

export function AdminBreadcrumbs({
  heading,
  action,
  onRefresh,
  refreshing,
  parentLinks,
}: AdminBreadcrumbsProps) {
  const { t } = useTranslate('admin');
  const parents = parentLinks ?? [{ name: t('nav.systemManagement') }];
  const links = [
    { name: t('nav.dashboard'), href: paths.dashboard.root },
    ...parents,
    { name: heading },
  ];

  return (
    <CustomBreadcrumbs
      heading={heading}
      links={links}
      action={headerActions(action, onRefresh, refreshing)}
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}

function headerActions(
  action: React.ReactNode,
  onRefresh: AdminBreadcrumbsProps['onRefresh'],
  refreshing: boolean | undefined
) {
  if (!onRefresh) return action;
  return (
    <Stack
      direction="row"
      spacing={1}
      useFlexGap
      flexWrap="wrap"
      alignItems="center"
      justifyContent="flex-end"
      sx={{ width: { xs: '100%', sm: 'auto' } }}
    >
      <PageRefreshButton onRefresh={onRefresh} loading={refreshing} />
      {action}
    </Stack>
  );
}
