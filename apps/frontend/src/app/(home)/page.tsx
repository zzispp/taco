import type { Metadata } from 'next';

import { HomeView } from 'src/sections/home/view';

// ----------------------------------------------------------------------

export const metadata: Metadata = {
  title: 'Hook | Backend Control Plane',
  description: 'Hook centralizes authentication, RBAC, API permissions, and menu governance.',
};

export default function Page() {
  return <HomeView />;
}
