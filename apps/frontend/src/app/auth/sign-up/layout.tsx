import { AuthSplitLayout } from 'src/widgets/auth-shell';

import { GuestGuard } from 'src/app/guards/guest-guard';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

export default function Layout({ children }: Props) {
  return (
    <GuestGuard>
      <AuthSplitLayout slotProps={{ section: { title: 'Create your Hook workspace' } }}>
        {children}
      </AuthSplitLayout>
    </GuestGuard>
  );
}
