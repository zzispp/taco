import { paths } from 'src/shared/routes/paths';

export type RootAuthState = Readonly<{
  loading: boolean;
  authenticated: boolean;
}>;

export function resolveRootTarget(state: RootAuthState) {
  if (state.loading) return null;
  return state.authenticated ? paths.dashboard.root : paths.auth.jwt.signIn;
}
