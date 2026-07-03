import type { AccountDrawerProps } from 'src/widgets/dashboard-shell/ui/account-drawer';

import { paths } from 'src/shared/routes/paths';
import { Iconify } from 'src/shared/ui/iconify';

// ----------------------------------------------------------------------

export const _account: AccountDrawerProps['data'] = [
  { label: 'Home', href: paths.home, icon: <Iconify icon="solar:home-angle-bold-duotone" /> },
  {
    label: 'Console',
    href: paths.dashboard.root,
    icon: <Iconify icon="solar:notes-bold-duotone" />,
  },
];
