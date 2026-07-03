import type { Metadata } from 'next';

import { CONFIG } from 'src/global-config';

import { View403 } from 'src/sections/error';

// ----------------------------------------------------------------------

export const metadata: Metadata = { title: `403 forbidden! | Error - ${CONFIG.appName}` };

export default function Page() {
  return <View403 />;
}
