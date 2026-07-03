import type { AccountDrawerProps } from './components/account-drawer';

import { paths } from 'src/routes/paths';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

export const _account: AccountDrawerProps['data'] = [
  { label: 'Home', href: paths.home, icon: <Iconify icon="solar:home-angle-bold-duotone" /> },
  {
    label: 'Console',
    href: paths.dashboard.root,
    icon: <Iconify icon="solar:notes-bold-duotone" />,
  },
];
