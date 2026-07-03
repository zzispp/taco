import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { Error404Page } from 'src/pages-layer/error-404';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `404 page not found! | Error - ${CONFIG.appName}` };

export default function Page() {
  return <Error404Page />;
}
