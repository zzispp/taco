import type { NavSectionProps } from 'src/components/nav-section';

import { paths } from 'src/routes/paths';

import { CONFIG } from 'src/global-config';

import { SvgColor } from 'src/components/svg-color';

// ----------------------------------------------------------------------

const icon = (name: string) => (
  <SvgColor src={`${CONFIG.assetsDir}/assets/icons/navbar/${name}.svg`} />
);

const ICONS = {
  user: icon('ic-user'),
  lock: icon('ic-lock'),
  label: icon('ic-label'),
  dashboard: icon('ic-dashboard'),
};

// ----------------------------------------------------------------------

export const navData: NavSectionProps['data'] = [
  {
    code: 'system_management',
    subheader: 'System Management',
    items: [
      { code: 'admin_users', title: 'Users', path: paths.dashboard.admin.users, icon: ICONS.user },
      { code: 'admin_roles', title: 'Roles', path: paths.dashboard.admin.roles, icon: ICONS.lock },
      { code: 'admin_apis', title: 'APIs', path: paths.dashboard.admin.apis, icon: ICONS.label },
      { code: 'admin_menus', title: 'Menus', path: paths.dashboard.admin.menus, icon: ICONS.dashboard },
    ],
  },
];
