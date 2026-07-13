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
    profile: `${ROOTS.DASHBOARD}/profile`,
    admin: {
      root: `${ROOTS.DASHBOARD}/admin`,
      users: `${ROOTS.DASHBOARD}/admin/users`,
      roles: `${ROOTS.DASHBOARD}/admin/roles`,
      menus: `${ROOTS.DASHBOARD}/admin/menus`,
      depts: `${ROOTS.DASHBOARD}/admin/depts`,
      posts: `${ROOTS.DASHBOARD}/admin/posts`,
      dicts: `${ROOTS.DASHBOARD}/admin/dicts`,
      configs: `${ROOTS.DASHBOARD}/admin/configs`,
      online: `${ROOTS.DASHBOARD}/admin/online`,
      jobs: `${ROOTS.DASHBOARD}/admin/jobs`,
      jobLogs: `${ROOTS.DASHBOARD}/admin/job-logs`,
    },
  },
};
