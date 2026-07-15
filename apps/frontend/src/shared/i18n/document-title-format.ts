import { CONFIG } from 'src/shared/config';

export function formatPageDocumentTitle(title: string, siteName = CONFIG.appName) {
  return `${title} | ${siteName}`;
}

export function formatErrorDocumentTitle(title: string, siteName = CONFIG.appName) {
  return `${title} | Error - ${siteName}`;
}

export function formatDashboardDocumentTitle(
  title: string,
  section: string,
  siteName = CONFIG.appName
) {
  return `${title} | ${section} - ${siteName}`;
}
