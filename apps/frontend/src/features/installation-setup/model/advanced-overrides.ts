import type { AdvancedSetupKey } from './form-values';

import { ADVANCED_SETUP_KEYS } from './form-values';

export type AdvancedDirtyFields = Readonly<Partial<Record<AdvancedSetupKey, boolean>>>;

export function getDirtyAdvancedKeys(
  dirtyFields: AdvancedDirtyFields | undefined,
  advancedOpen: boolean
): readonly AdvancedSetupKey[] {
  if (!advancedOpen) return [];

  return ADVANCED_SETUP_KEYS.filter((key) => Boolean(dirtyFields?.[key]));
}
