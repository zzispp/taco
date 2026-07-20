import type { Metadata } from 'next';

import { getDashboardPageMetadata } from 'src/shared/i18n/server';
import { resolveRouteLocale, type LocaleRouteParams } from 'src/shared/routes/locale-path';

import { DashboardHomePage } from 'src/pages-layer/dashboard-home';

// ----------------------------------------------------------------------

type PageProps = Readonly<{
  params: LocaleRouteParams;
}>;

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  return getDashboardPageMetadata(await resolveRouteLocale(params), 'nav.dashboard');
}

export default function Page() {
  return <DashboardHomePage />;
}
