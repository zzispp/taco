import { fileURLToPath } from 'node:url';
import { it, expect, describe } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';

const frontendRoot = fileURLToPath(new URL('../../../../', import.meta.url));

describe('admin widget FSD boundary', () => {
  it('does not keep admin business components in shared UI', () => {
    expect(existsSync(`${frontendRoot}src/shared/ui/admin`)).toBe(false);
  });

  it('consumes user-profile behavior through its public API', () => {
    const dialog = readFileSync(
      `${frontendRoot}src/widgets/account-profile-panel/ui/avatar-crop-dialog.tsx`,
      'utf8'
    );
    expect(dialog).not.toContain('src/features/user-profile/lib/');
  });
});
