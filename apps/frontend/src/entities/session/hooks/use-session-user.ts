import { resolveServerAssetUrl } from 'src/shared/lib/asset-url';
import { useSiteDisplay } from 'src/shared/config/site-display-context';

import { useAuthContext } from './use-auth-context';

// ----------------------------------------------------------------------

export function useSessionUser() {
  const { user: authUser } = useAuthContext();
  const { siteName } = useSiteDisplay();

  const user = {
    id: authUser?.user_id ?? 'anonymous-user',
    displayName:
      authUser?.displayName ?? authUser?.nick_name ?? authUser?.username ?? `${siteName} User`,
    email: authUser?.email ?? 'no-email@local.invalid',
    photoURL: authUser?.photoURL ?? resolveServerAssetUrl(authUser?.avatar),
    role: authUser?.roles.map((role) => role.role_key).join(', ') || 'none',
  };

  return { user };
}
