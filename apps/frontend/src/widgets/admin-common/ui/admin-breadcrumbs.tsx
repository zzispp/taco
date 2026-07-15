'use client';

import { paths } from 'src/shared/routes/paths';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { CustomBreadcrumbs } from 'src/shared/ui/custom-breadcrumbs';

type AdminBreadcrumbsProps = {
  heading: string;
  action?: React.ReactNode;
  parentLinks?: readonly AdminBreadcrumbLink[];
};

type AdminBreadcrumbLink = Readonly<{ name: string; href?: string }>;

export function AdminBreadcrumbs({ heading, action, parentLinks }: AdminBreadcrumbsProps) {
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
      action={action}
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}
