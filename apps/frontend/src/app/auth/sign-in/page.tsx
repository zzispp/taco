import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { JwtSignInView } from 'src/auth/view/jwt';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `Sign in | ${CONFIG.appName}` };

export default function Page() {
  return <JwtSignInView />;
}
