'use client';

import { useEffect } from 'react';

type SiteDocumentTitleProps = {
  title: string;
};

export function SiteDocumentTitle({ title }: SiteDocumentTitleProps) {
  useEffect(() => {
    document.title = title;
  }, [title]);

  return null;
}
