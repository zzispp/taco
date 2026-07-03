import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { SignUpPage } from 'src/pages-layer/sign-up';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign up | ${CONFIG.appName}` };

export default function Page() {
  return <SignUpPage />;
}
