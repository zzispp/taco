import { useAuthContext } from './use-auth-context';

// ----------------------------------------------------------------------

export function useSessionUser() {
  const { user: authUser } = useAuthContext();

  const user = {
    id: authUser?.user_id ?? 'hook-anonymous-user',
    displayName: authUser?.displayName ?? authUser?.nick_name ?? authUser?.username ?? 'Hook User',
    email: authUser?.email ?? 'no-email@hook.local',
    photoURL: authUser?.photoURL ?? authUser?.avatar ?? '',
    role: authUser?.roles.map((role) => role.role_key).join(', ') || 'none',
  };

  return { user };
}
