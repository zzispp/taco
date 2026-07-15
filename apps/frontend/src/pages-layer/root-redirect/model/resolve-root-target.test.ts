import { it, expect, describe } from 'vitest';

import { paths } from 'src/shared/routes/paths';

import { resolveRootTarget } from './resolve-root-target';

describe('root redirect target', () => {
  it('waits until authentication restoration completes', () => {
    expect(resolveRootTarget({ loading: true, authenticated: false })).toBeNull();
  });

  it('sends authenticated users to the dashboard', () => {
    expect(resolveRootTarget({ loading: false, authenticated: true })).toBe(paths.dashboard.root);
  });

  it('sends unauthenticated users to sign in', () => {
    expect(resolveRootTarget({ loading: false, authenticated: false })).toBe(paths.auth.jwt.signIn);
  });
});
