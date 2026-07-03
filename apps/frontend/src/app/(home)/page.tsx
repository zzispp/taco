import type { Metadata } from 'next';

import { HomePage } from 'src/pages-layer/home';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: 'Hook | Backend Control Plane',
  description: 'Hook centralizes authentication, RBAC, API permissions, and menu governance.',
};

export default function Page() {
  return <HomePage />;
}
