import { ESLint } from 'eslint';
import { fileURLToPath } from 'node:url';
import { it, expect, describe } from 'vitest';

const frontendRoot = fileURLToPath(new URL('../../', import.meta.url));
const sessionFixture = `${frontendRoot}src/entities/session/model/boundary-fixture.ts`;

describe('entity slice FSD boundary', () => {
  it('rejects both value and type imports from another entity slice', async () => {
    const eslint = new ESLint({ cwd: frontendRoot });
    const results = await Promise.all([
      eslint.lintText("import type { RoleSummary } from 'src/entities/role';\n\nexport type SessionRole = RoleSummary;", {
        filePath: sessionFixture,
      }),
      eslint.lintText("import { roleEndpoints } from 'src/entities/role';\n\nexport const endpoint = roleEndpoints.roles;", {
        filePath: sessionFixture,
      }),
    ]);

    for (const [result] of results) {
      const boundaryErrors = result.messages.filter(
        (message) => message.ruleId === 'import/no-restricted-paths'
      );
      expect(boundaryErrors).toHaveLength(1);
    }
  });

  it('allows imports within one entity slice', async () => {
    const eslint = new ESLint({ cwd: frontendRoot });
    const [result] = await eslint.lintText(
      "import type { SessionUser } from 'src/entities/session';\n\nexport type SessionValue = SessionUser;",
      { filePath: sessionFixture }
    );

    expect(result.messages.filter((message) => message.ruleId === 'import/no-restricted-paths')).toEqual([]);
  });
});
