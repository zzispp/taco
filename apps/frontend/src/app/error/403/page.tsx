import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { Error403Page } from 'src/pages-layer/error-403';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `403 forbidden! | Error - ${CONFIG.appName}` };

export default function Page() {
  return <Error403Page />;
}
