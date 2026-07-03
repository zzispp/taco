import { useAuthContext } from './use-auth-context';

// ----------------------------------------------------------------------

export function useMockedUser() {
  const { user: authUser } = useAuthContext();

  const user = {
    id: authUser?.id ?? 'hook-anonymous-user',
    displayName: authUser?.displayName ?? authUser?.username ?? 'Hook User',
    email: authUser?.email ?? 'no-email@hook.local',
    photoURL: authUser?.photoURL ?? '',
    role: authUser?.role ?? 'admin',
  };

  return { user };
}
