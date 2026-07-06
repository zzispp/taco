'use client';

import { paths } from 'src/shared/routes/paths';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { CustomBreadcrumbs } from 'src/shared/ui/custom-breadcrumbs';

type AdminBreadcrumbsProps = {
  heading: string;
  action?: React.ReactNode;
};

export function AdminBreadcrumbs({ heading, action }: AdminBreadcrumbsProps) {
  const { t } = useTranslate('admin');

  return (
    <CustomBreadcrumbs
      heading={heading}
      links={[
        { name: t('nav.dashboard'), href: paths.dashboard.root },
        { name: t('nav.systemManagement') },
        { name: heading },
      ]}
      action={action}
      sx={{ mb: { xs: 3, md: 5 } }}
    />
  );
}
