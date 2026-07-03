import { CONFIG } from 'src/shared/config';

import { DashboardLayout } from 'src/widgets/dashboard-shell';

import { AuthGuard } from 'src/app/guards/auth-guard';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

export default function Layout({ children }: Props) {
  if (CONFIG.auth.skip) {
    return <DashboardLayout>{children}</DashboardLayout>;
  }

  return (
    <AuthGuard>
      <DashboardLayout>{children}</DashboardLayout>
    </AuthGuard>
  );
}
