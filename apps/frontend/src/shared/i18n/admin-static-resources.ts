import enAdmin from './langs/en/admin.json';
import cnAdmin from './langs/cn/admin.json';
import twAdmin from './langs/tw/admin.json';
import { mergeAdminResources } from './admin-resources';
import enAdminNotice from './langs/en/admin-notice.json';
import cnAdminNotice from './langs/cn/admin-notice.json';
import twAdminNotice from './langs/tw/admin-notice.json';
import enAdminProfile from './langs/en/admin-profile.json';
import cnAdminProfile from './langs/cn/admin-profile.json';
import twAdminProfile from './langs/tw/admin-profile.json';
import enAdminDashboard from './langs/en/admin-dashboard.json';
import cnAdminDashboard from './langs/cn/admin-dashboard.json';
import twAdminDashboard from './langs/tw/admin-dashboard.json';
import enAdminNavigation from './langs/en/admin-navigation.json';
import cnAdminNavigation from './langs/cn/admin-navigation.json';
import twAdminNavigation from './langs/tw/admin-navigation.json';
import enAdminAccessControl from './langs/en/admin-access-control.json';
import cnAdminAccessControl from './langs/cn/admin-access-control.json';
import twAdminAccessControl from './langs/tw/admin-access-control.json';
import enAdminOnlineSessions from './langs/en/admin-online-sessions.json';
import cnAdminOnlineSessions from './langs/cn/admin-online-sessions.json';
import twAdminOnlineSessions from './langs/tw/admin-online-sessions.json';

export const staticAdminResources = {
  cn: mergeAdminResources(
    cnAdmin,
    cnAdminNavigation,
    cnAdminDashboard,
    cnAdminAccessControl,
    cnAdminProfile,
    cnAdminOnlineSessions,
    cnAdminNotice
  ),
  en: mergeAdminResources(
    enAdmin,
    enAdminNavigation,
    enAdminDashboard,
    enAdminAccessControl,
    enAdminProfile,
    enAdminOnlineSessions,
    enAdminNotice
  ),
  tw: mergeAdminResources(
    twAdmin,
    twAdminNavigation,
    twAdminDashboard,
    twAdminAccessControl,
    twAdminProfile,
    twAdminOnlineSessions,
    twAdminNotice
  ),
} as const;
