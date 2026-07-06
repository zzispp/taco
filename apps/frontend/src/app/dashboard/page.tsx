import type { Metadata } from 'next';

import { redirect } from 'next/navigation';

import { paths } from 'src/shared/routes/paths';
import { formatPageDocumentTitle } from 'src/shared/i18n/document-title-format';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: formatPageDocumentTitle('Dashboard') };

export default function Page() {
  redirect(paths.dashboard.admin.users);
}
