import type { RegistryStatus } from 'src/entities/scheduler';

import { REGISTRY_STATUS } from 'src/entities/scheduler';

export type SchedulerJobCapabilities = Readonly<{
  editable: boolean;
  runnable: boolean;
}>;

const CAPABILITIES = {
  [REGISTRY_STATUS.OK]: { editable: true, runnable: true },
  [REGISTRY_STATUS.INVALID_PARAMS]: { editable: true, runnable: false },
  [REGISTRY_STATUS.MISSING]: { editable: false, runnable: false },
  [REGISTRY_STATUS.REPEATABLE_MISMATCH]: { editable: false, runnable: false },
} as const satisfies Record<RegistryStatus, SchedulerJobCapabilities>;

export function schedulerJobCapabilities(status: RegistryStatus): SchedulerJobCapabilities {
  return CAPABILITIES[status];
}
