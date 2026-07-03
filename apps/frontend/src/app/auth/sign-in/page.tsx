import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { SignInPage } from 'src/pages-layer/sign-in';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign in | ${CONFIG.appName}` };

export default function Page() {
  return <SignInPage />;
}
