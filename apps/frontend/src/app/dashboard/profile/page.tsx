import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';

import { ProfilePage } from 'src/pages-layer/profile';

// ----------------------------------------------------------------------

export function generateMetadata(): Promise<Metadata> {
  return getDashboardPageMetadata('profile.personalCenter');
}

export default function Page() {
  return <ProfilePage />;
}
