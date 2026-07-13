import type {
  JobStatus,
  ParamWidget,
  JobLogStatus,
  MisfirePolicy,
  RegistryStatus,
  ConcurrentPolicy,
  SchedulerTriggerType,
  CapturedBytesEncoding,
  HttpExecutionFailureCode,
  SchedulerRuntimeErrorCode,
} from './constants';

export type ParamSchema =
  | {
      type: 'object';
      properties: Record<string, ParamSchema>;
      required: string[];
      additional_properties: boolean;
    }
  | { type: 'string'; format: string | null; pattern: string | null; enum_values: string[] }
  | { type: 'number'; min: number | null; max: number | null }
  | { type: 'boolean' }
  | { type: 'record'; key: ParamSchema; value: ParamSchema }
  | { type: 'array'; items: ParamSchema };

export type ParamCondition = { path: string; values: unknown[] };

export type ParamFieldSpec = {
  path: string;
  label: string;
  widget: ParamWidget;
  placeholder: string | null;
  help: string | null;
  options: string[];
  disabled_when: ParamCondition | null;
};

export type TaskParamFormSpec = {
  schema_version: number;
  schema: ParamSchema;
  ui: { fields: ParamFieldSpec[] };
};

export type ImportableTask = {
  task_key: string;
  name: string;
  group: string;
  group_label: string;
  description: string;
  repeatable: boolean;
  default_params: Record<string, unknown>;
  param_form: TaskParamFormSpec;
};

export type SchedulerJob = {
  job_id: string;
  job_name: string;
  job_group: string;
  task_key: string;
  task_params: Record<string, unknown>;
  params_schema_version: number;
  repeatable: boolean;
  invoke_target: string;
  cron_expression: string;
  misfire_policy: MisfirePolicy;
  concurrent: ConcurrentPolicy;
  status: JobStatus;
  registry_status: RegistryStatus;
  param_form: TaskParamFormSpec | null;
  schedule_revision: number;
  next_run_at: string | null;
  runtime_error: SchedulerRuntimeError | null;
  create_by: string;
  create_time: string;
  update_by: string;
  update_time: string | null;
  remark: string | null;
};

export type SchedulerJobLog = {
  execution_id: string;
  job_id: string;
  job_name: string;
  job_group: string;
  task_key: string;
  invoke_target: string;
  job_message: string;
  trigger_type: SchedulerTriggerType;
  scheduled_at: string;
  status: JobLogStatus;
  exception_info: string | null;
  start_time: string | null;
  end_time: string;
  create_time: string;
  has_detail: boolean;
};

export type SchedulerJobLogQuery = {
  job_name?: string;
  job_group?: string;
  status?: JobLogStatus;
  trigger_type?: SchedulerTriggerType;
  begin_time?: string;
  end_time?: string;
};

export type CapturedBytes = {
  encoding: CapturedBytesEncoding;
  content: string;
  byte_length: number;
};

export type CapturedHeader = {
  name: string;
  value: CapturedBytes;
};

export type HttpExecutionRequest = {
  method: string;
  url: string;
  headers: readonly CapturedHeader[];
  body: CapturedBytes | null;
};

export type HttpExecutionResponse = {
  status: number;
  final_url: string;
  headers: readonly CapturedHeader[];
  body: CapturedBytes | null;
};

export type HttpExecutionFailure = {
  code: HttpExecutionFailureCode;
};

export type HttpExecutionDetailPayload = {
  duration_ms: number;
  request: HttpExecutionRequest;
  response: HttpExecutionResponse | null;
  failure: HttpExecutionFailure | null;
};

export type ExecutionDetailEnvelope = {
  kind: string;
  schema_version: number;
  payload: unknown;
};

export type SchedulerExecutionDetail = SchedulerJobLog & {
  job_revision: number;
  requested_by: string | null;
  task_params: Record<string, unknown>;
  detail: ExecutionDetailEnvelope | null;
};

export type SchedulerRuntimeError = {
  code: SchedulerRuntimeErrorCode;
  message: string;
  occurred_at: string;
};

export type RunJobResponse = {
  accepted: true;
  execution_id: string;
};

export type ImportJobInput = {
  task_key: string;
  job_name: string;
  job_group: string;
  cron_expression: string;
  misfire_policy: MisfirePolicy;
  concurrent: ConcurrentPolicy;
  task_params: Record<string, unknown>;
  remark?: string | null;
};

export type ReplaceJobInput = Omit<ImportJobInput, 'task_key'>;
