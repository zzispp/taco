'use client';

import type { LinkProps } from 'next/link';

import NextLink from 'next/link';
import { forwardRef } from 'react';

import { usePathname } from 'src/shared/routes/hooks/use-pathname';
import { localizePath, requireLangCode, localeFromPathname } from 'src/shared/routes/locale-path';

type RouterLinkProps = LinkProps &
  Omit<React.AnchorHTMLAttributes<HTMLAnchorElement>, keyof LinkProps>;

const RouterLink = forwardRef<HTMLAnchorElement, RouterLinkProps>(function RouterLink(
  { href, ...props },
  ref
) {
  const pathname = usePathname();
  const localizedHref = typeof href === 'string' ? localizedLinkHref(pathname, href) : href;

  return <NextLink ref={ref} href={localizedHref} {...props} />;
});

RouterLink.displayName = 'RouterLink';

export { RouterLink };

function localizedLinkHref(pathname: string, href: string) {
  if (localeFromPathname(href)) return href;

  const locale = localeFromPathname(pathname);
  if (!locale) {
    throw new Error(`Cannot localize internal link "${href}" outside a locale route: ${pathname}`);
  }

  return localizePath(requireLangCode(locale), href);
}
