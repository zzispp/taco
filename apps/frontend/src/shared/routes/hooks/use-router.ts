import NProgress from 'nprogress';
import { useMemo, useCallback } from 'react';
import { isEqualPath } from 'minimal-shared/utils';
import { useRouter as useNextRouter } from 'next/navigation';

import { localizePath } from 'src/shared/routes/locale-path';

import { useLocale } from './use-locale';

// ----------------------------------------------------------------------

/**
 * Customized useRouter hook with NProgress integration.
 */

export function useRouter() {
  const nextRouter = useNextRouter();
  const locale = useLocale();

  const localizeHref = useCallback((href: string) => localizePath(locale, href), [locale]);

  const push: ReturnType<typeof useNextRouter>['push'] = useCallback(
    (href, options) => {
      const localizedHref = localizeHref(href);
      if (
        typeof window !== 'undefined' &&
        !isEqualPath(localizedHref, window.location.href, { deep: false })
      ) {
        NProgress.start();
      }
      nextRouter.push(localizedHref, options);
    },
    [localizeHref, nextRouter]
  );

  const replace: ReturnType<typeof useNextRouter>['replace'] = useCallback(
    (href, options) => {
      const localizedHref = localizeHref(href);
      if (
        typeof window !== 'undefined' &&
        !isEqualPath(localizedHref, window.location.href, { deep: false })
      ) {
        NProgress.start();
      }
      nextRouter.replace(localizedHref, options);
    },
    [localizeHref, nextRouter]
  );

  const router = useMemo(
    () => ({
      ...nextRouter,
      push,
      replace,
    }),
    [nextRouter, push, replace]
  );

  return router;
}
