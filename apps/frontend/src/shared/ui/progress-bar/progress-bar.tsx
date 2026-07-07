'use client';

import './styles.css';

import NProgress from 'nprogress';
import { useRef, useEffect, useCallback } from 'react';

import { usePathname } from 'src/shared/routes/hooks';

import { clickedAnchor, isValidAnchor, patchHistoryMethod, startProgressForUrl } from './navigation-progress';

const COMPLETE_DELAY_MS = 100;

function useProgressBar() {
  const pathname = usePathname();
  const currentUrlRef = useRef<string>('');

  useEffect(() => {
    currentUrlRef.current = window.location.href;
  }, []);

  const handleNavigation = useCallback((newUrl: string) => {
    currentUrlRef.current = startProgressForUrl(currentUrlRef.current, newUrl);
  }, []);

  useEffect(() => {
    const handleClickAnchor = (event: MouseEvent) => {
      const anchor = clickedAnchor(event);
      if (anchor && isValidAnchor(anchor)) handleNavigation(anchor.href);
    };
    const handlePopState = () => handleNavigation(window.location.href);

    patchHistoryMethod({ method: 'pushState', onNavigate: handleNavigation });
    patchHistoryMethod({ method: 'replaceState', onNavigate: handleNavigation });
    document.addEventListener('click', handleClickAnchor);
    window.addEventListener('popstate', handlePopState);

    return () => {
      document.removeEventListener('click', handleClickAnchor);
      window.removeEventListener('popstate', handlePopState);
    };
  }, [handleNavigation]);

  useEffect(() => {
    const timeout = setTimeout(() => NProgress.done(), COMPLETE_DELAY_MS);
    return () => clearTimeout(timeout);
  }, [pathname]);
}

export function ProgressBar() {
  useEffect(() => {
    NProgress.configure({ showSpinner: false });
    return () => {
      NProgress.done();
    };
  }, []);

  useProgressBar();

  return null;
}
