import { requireLangCode } from 'src/shared/routes/locale-path';

import { usePathname } from './use-pathname';

export function useLocale() {
  const pathname = usePathname();
  const locale = pathname.split('/').find(Boolean);

  return requireLangCode(locale);
}
