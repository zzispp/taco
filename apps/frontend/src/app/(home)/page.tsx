import type { Metadata } from 'next';

import { formatHomeDocumentTitle } from 'src/shared/i18n/document-title-format';

import { HomePage } from 'src/pages-layer/home';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: formatHomeDocumentTitle(),
  description:
    'Backend control plane for authentication, RBAC, API permissions, and menu governance.',
};

export default function Page() {
  return <HomePage />;
}
