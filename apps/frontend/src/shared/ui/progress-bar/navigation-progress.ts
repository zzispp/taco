import NProgress from 'nprogress';
import { isEqualPath } from 'minimal-shared/utils';

export function isValidAnchor(element: HTMLAnchorElement): boolean {
  const href = element.getAttribute('href')?.trim() ?? '';
  const target = element.getAttribute('target');
  const rel = element.getAttribute('rel');

  return (
    href.startsWith('/') &&
    target !== '_blank' &&
    (!rel || !['noopener', 'noreferrer'].some((value) => rel.includes(value)))
  );
}

export function startProgressForUrl(currentUrl: string, newUrl: string) {
  if (!newUrl || isEqualPath(newUrl, currentUrl, { deep: false })) {
    return currentUrl;
  }
  NProgress.start();
  return newUrl;
}

export function clickedAnchor(event: MouseEvent) {
  const target = event.target as HTMLElement;
  return target.closest('a[href]') as HTMLAnchorElement | null;
}

export function patchHistoryMethod(options: PatchHistoryMethodOptions) {
  const originalMethod = window.history[options.method];
  window.history[options.method] = new Proxy(originalMethod, {
    apply: (target, thisArg, args: [data: unknown, unused: string, url?: string | URL | null]) => {
      const newUrl = args[2];
      if (typeof newUrl === 'string') {
        options.onNavigate(new URL(newUrl, window.location.origin).href);
      }
      return target.apply(thisArg, args);
    },
  });
}

type PatchHistoryMethodOptions = {
  method: 'pushState' | 'replaceState';
  onNavigate: (url: string) => void;
};
