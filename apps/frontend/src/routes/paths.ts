const ROOTS = {
  AUTH: '/auth',
  DASHBOARD: '/dashboard',
};

// ----------------------------------------------------------------------

export const paths = {
  home: '/',
  page403: '/error/403',
  page404: '/error/404',
  page500: '/error/500',
  auth: {
    jwt: {
      signIn: `${ROOTS.AUTH}/sign-in`,
      signUp: `${ROOTS.AUTH}/sign-up`,
    },
  },
  dashboard: {
    root: ROOTS.DASHBOARD,
    admin: {
      root: `${ROOTS.DASHBOARD}/admin`,
      users: `${ROOTS.DASHBOARD}/admin/users`,
      roles: `${ROOTS.DASHBOARD}/admin/roles`,
      apis: `${ROOTS.DASHBOARD}/admin/apis`,
      menus: `${ROOTS.DASHBOARD}/admin/menus`,
    },
  },
};
