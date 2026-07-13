import type { ParamDraft } from './param-draft';
import type {
  SchedulerJob,
  ImportableTask,
  ImportJobInput,
  ReplaceJobInput,
  TaskParamFormSpec,
} from 'src/entities/scheduler';

import * as z from 'zod';

import {
  MISFIRE_POLICY,
  CONCURRENT_POLICY,
  DEFAULT_SCHEDULER_JOB_GROUP,
} from 'src/entities/scheduler';

import { compileParamSchema } from './param-schema';
import { createParamDraft, materializeParamDraft } from './param-draft';

export type JobFormState = Omit<ImportJobInput, 'task_params'> & {
  paramDraft: ParamDraft;
};

export const DEFAULT_CRON_EXPRESSION = '0 0/5 * * * ? *';

const jobSchema = z.object({
  job_name: z.string().trim().min(1),
  job_group: z.string().trim().min(1),
  cron_expression: z.string().trim().min(1),
  misfire_policy: z.enum([MISFIRE_POLICY.FIRE_ONCE, MISFIRE_POLICY.DO_NOTHING]),
  concurrent: z.enum([CONCURRENT_POLICY.ALLOW, CONCURRENT_POLICY.DISALLOW]),
});

export function createJobForm(
  task: ImportableTask | null | undefined,
  job: SchedulerJob | null | undefined,
  createId: () => string
): JobFormState {
  if (job) return createJobFormFromJob(job, createId);
  if (task) return createJobFormFromTask(task, createId);
  return {
    task_key: '',
    job_name: '',
    job_group: DEFAULT_SCHEDULER_JOB_GROUP,
    cron_expression: DEFAULT_CRON_EXPRESSION,
    misfire_policy: MISFIRE_POLICY.DO_NOTHING,
    concurrent: CONCURRENT_POLICY.DISALLOW,
    paramDraft: createParamDraft({}, [], createId),
    remark: '',
  };
}

function createJobFormFromJob(job: SchedulerJob, createId: () => string): JobFormState {
  const fields = job.param_form?.ui.fields ?? [];
  return {
    task_key: job.task_key,
    job_name: job.job_name,
    job_group: job.job_group,
    cron_expression: job.cron_expression,
    misfire_policy: job.misfire_policy,
    concurrent: job.concurrent,
    paramDraft: createParamDraft(job.task_params, fields, createId),
    remark: job.remark ?? '',
  };
}

function createJobFormFromTask(task: ImportableTask, createId: () => string): JobFormState {
  return {
    task_key: task.task_key,
    job_name: task.name,
    job_group: task.group,
    cron_expression: DEFAULT_CRON_EXPRESSION,
    misfire_policy: MISFIRE_POLICY.DO_NOTHING,
    concurrent: CONCURRENT_POLICY.DISALLOW,
    paramDraft: createParamDraft(task.default_params, task.param_form.ui.fields, createId),
    remark: '',
  };
}

export function materializeJobInput(
  form: JobFormState,
  paramForm: TaskParamFormSpec
): ImportJobInput {
  jobSchema.parse(form);
  const taskParams = materializeParamDraft(form.paramDraft, paramForm.ui.fields);
  compileParamSchema(paramForm.schema).parse(taskParams);
  return {
    task_key: form.task_key,
    job_name: form.job_name,
    job_group: form.job_group,
    cron_expression: form.cron_expression,
    misfire_policy: form.misfire_policy,
    concurrent: form.concurrent,
    task_params: taskParams,
    remark: form.remark,
  };
}

export function toReplaceJobInput(input: ImportJobInput): ReplaceJobInput {
  return {
    job_name: input.job_name,
    job_group: input.job_group,
    cron_expression: input.cron_expression,
    misfire_policy: input.misfire_policy,
    concurrent: input.concurrent,
    task_params: input.task_params,
    remark: input.remark,
  };
}

export function emptyParamForm(): TaskParamFormSpec {
  return {
    schema_version: 1,
    schema: {
      type: 'object',
      properties: {},
      required: [],
      additional_properties: false,
    },
    ui: { fields: [] },
  };
}
