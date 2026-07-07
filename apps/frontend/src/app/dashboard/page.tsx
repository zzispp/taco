import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { DashboardHomePage } from 'src/pages-layer/dashboard-home';

// ----------------------------------------------------------------------

export async function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('nav.dashboard');
}

export default function Page() {
  return <DashboardHomePage />;
}
