import { paths } from 'src/shared/routes/paths';

const SECTION_KEY_BY_CODE: Record<string, string> = {
  account: 'nav.account',
  resources: 'nav.resources',
  system_management: 'nav.systemManagement',
  system_monitor: 'nav.systemMonitor',
  '1': 'nav.systemManagement',
  '3': 'nav.systemMonitor',
  '4': 'nav.overview',
  '5': 'nav.fileManagement',
};

const ITEM_KEY_BY_CODE: Record<string, string> = {
  '2': 'nav.dashboard',
  '100': 'nav.users',
  '101': 'nav.roles',
  '102': 'nav.menus',
  '103': 'nav.depts',
  '104': 'nav.posts',
  '105': 'nav.dicts',
  '106': 'nav.configs',
  '107': 'nav.online',
  '108': 'nav.jobs',
  '109': 'nav.jobLogs',
  '110': 'nav.notices',
  '111': 'nav.logManagement',
  '112': 'nav.operationLogs',
  '113': 'nav.loginLogs',
  '114': 'nav.systemLogs',
};

const ITEM_KEY_BY_PATH: Record<string, string> = {
  [paths.dashboard.root]: 'nav.dashboard',
  [paths.dashboard.overview]: 'nav.overview',
  [paths.dashboard.files]: 'nav.fileManagement',
  [paths.dashboard.file]: 'nav.fileOverview',
  [paths.dashboard.fileManager]: 'nav.fileManager',
  [paths.dashboard.fileSpaces]: 'nav.fileSpaces',
  [paths.dashboard.admin.root]: 'nav.systemManagement',
  [paths.dashboard.monitor]: 'nav.systemMonitor',
  [paths.dashboard.admin.users]: 'nav.users',
  [paths.dashboard.admin.roles]: 'nav.roles',
  [paths.dashboard.admin.menus]: 'nav.menus',
  [paths.dashboard.admin.depts]: 'nav.depts',
  [paths.dashboard.admin.posts]: 'nav.posts',
  [paths.dashboard.admin.dicts]: 'nav.dicts',
  [paths.dashboard.admin.configs]: 'nav.configs',
  [paths.dashboard.admin.online]: 'nav.online',
  [paths.dashboard.admin.jobs]: 'nav.jobs',
  [paths.dashboard.admin.jobLogs]: 'nav.jobLogs',
  [paths.dashboard.admin.notices]: 'nav.notices',
  [paths.dashboard.monitorLogs.root]: 'nav.logManagement',
  [paths.dashboard.monitorLogs.operationLogs]: 'nav.operationLogs',
  [paths.dashboard.monitorLogs.loginLogs]: 'nav.loginLogs',
  [paths.dashboard.monitorLogs.systemLogs]: 'nav.systemLogs',
};

export function systemMenuSectionTranslationKey(code?: string) {
  return code ? SECTION_KEY_BY_CODE[code] : undefined;
}

export function systemMenuItemTranslationKey(code: string | undefined, path: string) {
  return (code ? ITEM_KEY_BY_CODE[code] : undefined) ?? ITEM_KEY_BY_PATH[path];
}
