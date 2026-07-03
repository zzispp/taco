import type { NavMainProps } from './main/nav/types';

import { paths } from 'src/routes/paths';

import { Iconify } from 'src/components/iconify';

// ----------------------------------------------------------------------

export const navData: NavMainProps['data'] = [
  {
    title: 'Home',
    path: paths.home,
    icon: <Iconify width={22} icon="solar:home-angle-bold-duotone" />,
  },
  {
    title: 'Console',
    path: paths.dashboard.root,
    icon: <Iconify width={22} icon="solar:notes-bold-duotone" />,
  },
];
