import { getCurrentLang } from '../locales-config';
import { localeFromPathname } from '../../routes/locale-path';

// ----------------------------------------------------------------------

export function formatNumberLocale(pathname?: string) {
  const routePathname = pathname ?? (typeof window === 'undefined' ? '' : window.location.pathname);
  const currentLang = getCurrentLang(localeFromPathname(routePathname));

  return {
    code: currentLang?.numberFormat.code,
  };
}
