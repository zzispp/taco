import type { Metadata } from 'next';

import { CONFIG } from 'src/shared/config';

import { Error500Page } from 'src/pages-layer/error-500';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: `500 Internal server error! | Error - ${CONFIG.appName}`,
};

export default function Page() {
  return <Error500Page />;
}
