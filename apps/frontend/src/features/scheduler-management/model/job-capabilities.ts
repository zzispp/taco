import type {
  SchedulerJob,
  RegistryStatus,
  SchedulerJobLifecycleCapabilities,
} from 'src/entities/scheduler';

import { REGISTRY_STATUS } from 'src/entities/scheduler';

export type SchedulerJobCapabilities = Readonly<{
  editable: boolean;
  runnable: boolean;
  canDisable: boolean;
  canDelete: boolean;
  canEditExecutionPolicy: boolean;
}>;

const REGISTRY_CAPABILITIES = {
  [REGISTRY_STATUS.OK]: { editable: true, runnable: true },
  [REGISTRY_STATUS.INVALID_PARAMS]: { editable: true, runnable: false },
  [REGISTRY_STATUS.MISSING]: { editable: false, runnable: false },
  [REGISTRY_STATUS.REPEATABLE_MISMATCH]: { editable: false, runnable: false },
} as const satisfies Record<
  RegistryStatus,
  Pick<SchedulerJobCapabilities, 'editable' | 'runnable'>
>;

export function schedulerJobCapabilities(
  status: RegistryStatus,
  lifecycle: SchedulerJobLifecycleCapabilities
): SchedulerJobCapabilities {
  return {
    ...REGISTRY_CAPABILITIES[status],
    canDisable: lifecycle.can_disable,
    canDelete: lifecycle.can_delete,
    canEditExecutionPolicy: lifecycle.can_edit_execution_policy,
  };
}

export function selectableSchedulerJobIds(jobs: readonly SchedulerJob[], canRemove: boolean) {
  if (!canRemove) return [];
  return jobs
    .filter((job) => schedulerJobCapabilities(job.registry_status, job.capabilities).canDelete)
    .map((job) => job.job_id);
}
