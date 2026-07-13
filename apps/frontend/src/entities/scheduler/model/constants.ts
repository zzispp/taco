type ValueOf<T> = T[keyof T];

export const DEFAULT_SCHEDULER_JOB_GROUP = 'SYSTEM';

export const SCHEDULER_PERMISSION = {
  JOB_QUERY: 'system:job:query',
  JOB_IMPORT: 'system:job:import',
  JOB_EDIT: 'system:job:edit',
  JOB_REMOVE: 'system:job:remove',
  JOB_EXPORT: 'system:job:export',
  JOB_RUN: 'system:job:run',
  JOB_CHANGE_STATUS: 'system:job:changeStatus',
  JOB_LOG_QUERY: 'system:job:log:query',
  JOB_LOG_DETAIL: 'system:job:log:detail',
  JOB_LOG_REMOVE: 'system:job:log:remove',
  JOB_LOG_EXPORT: 'system:job:log:export',
} as const;

export type SchedulerPermission = ValueOf<typeof SCHEDULER_PERMISSION>;

export const REGISTRY_STATUS = {
  OK: 'ok',
  MISSING: 'missing',
  REPEATABLE_MISMATCH: 'repeatable_mismatch',
  INVALID_PARAMS: 'invalid_params',
} as const;

export type RegistryStatus = ValueOf<typeof REGISTRY_STATUS>;

export const RUNTIME_ERROR_CODE = {
  TASK_MISSING: 'task_missing',
  REPEATABLE_MISMATCH: 'repeatable_mismatch',
  INVALID_PARAMS: 'invalid_params',
  INVALID_CRON: 'invalid_cron',
} as const;

export type SchedulerRuntimeErrorCode = ValueOf<typeof RUNTIME_ERROR_CODE>;

export const JOB_STATUS = {
  NORMAL: '0',
  PAUSED: '1',
} as const;

export type JobStatus = ValueOf<typeof JOB_STATUS>;

export const CONCURRENT_POLICY = {
  ALLOW: '0',
  DISALLOW: '1',
} as const;

export type ConcurrentPolicy = ValueOf<typeof CONCURRENT_POLICY>;

export const MISFIRE_POLICY = {
  FIRE_ONCE: '2',
  DO_NOTHING: '3',
} as const;

export type MisfirePolicy = ValueOf<typeof MISFIRE_POLICY>;

export const JOB_LOG_STATUS = {
  SUCCESS: '0',
  FAILED: '1',
  SKIPPED: '2',
  INTERRUPTED: '3',
} as const;

export type JobLogStatus = ValueOf<typeof JOB_LOG_STATUS>;

export const SCHEDULER_TRIGGER_TYPE = {
  SCHEDULED: 'scheduled',
  MANUAL: 'manual',
  MISFIRE: 'misfire',
} as const;

export type SchedulerTriggerType = ValueOf<typeof SCHEDULER_TRIGGER_TYPE>;

export const EXECUTION_DETAIL_KIND = {
  HTTP_EXCHANGE: 'http_exchange',
} as const;

export const HTTP_EXECUTION_DETAIL_SCHEMA_VERSION = 1;

export const CAPTURED_BYTES_ENCODING = {
  UTF8: 'utf8',
  BASE64: 'base64',
} as const;

export type CapturedBytesEncoding = ValueOf<typeof CAPTURED_BYTES_ENCODING>;

export const HTTP_EXECUTION_FAILURE_CODE = {
  REQUEST_BUILD: 'request_build',
  TIMEOUT: 'timeout',
  CONNECT: 'connect',
  REQUEST: 'request',
  RESPONSE_BODY: 'response_body',
  HTTP_STATUS: 'http_status',
} as const;

export type HttpExecutionFailureCode = ValueOf<typeof HTTP_EXECUTION_FAILURE_CODE>;

export const PARAM_WIDGET = {
  TEXT: 'text',
  NUMBER: 'number',
  SELECT: 'select',
  TEXTAREA: 'textarea',
  KEY_VALUE: 'key_value',
  SWITCH: 'switch',
  JSON_EDITOR: 'json_editor',
} as const;

export type ParamWidget = ValueOf<typeof PARAM_WIDGET>;

export const registryStatusTranslationKeys = {
  [REGISTRY_STATUS.OK]: 'registryStatusValues.ok',
  [REGISTRY_STATUS.MISSING]: 'registryStatusValues.missing',
  [REGISTRY_STATUS.REPEATABLE_MISMATCH]: 'registryStatusValues.repeatableMismatch',
  [REGISTRY_STATUS.INVALID_PARAMS]: 'registryStatusValues.invalidParams',
} as const satisfies Record<RegistryStatus, string>;

export const runtimeErrorTranslationKeys = {
  [RUNTIME_ERROR_CODE.TASK_MISSING]: 'runtimeErrorValues.taskMissing',
  [RUNTIME_ERROR_CODE.REPEATABLE_MISMATCH]: 'runtimeErrorValues.repeatableMismatch',
  [RUNTIME_ERROR_CODE.INVALID_PARAMS]: 'runtimeErrorValues.invalidParams',
  [RUNTIME_ERROR_CODE.INVALID_CRON]: 'runtimeErrorValues.invalidCron',
} as const satisfies Record<SchedulerRuntimeErrorCode, string>;

export const jobStatusTranslationKeys = {
  [JOB_STATUS.NORMAL]: 'jobStatus.normal',
  [JOB_STATUS.PAUSED]: 'jobStatus.paused',
} as const satisfies Record<JobStatus, string>;

export const concurrentPolicyTranslationKeys = {
  [CONCURRENT_POLICY.ALLOW]: 'concurrentPolicy.allow',
  [CONCURRENT_POLICY.DISALLOW]: 'concurrentPolicy.disallow',
} as const satisfies Record<ConcurrentPolicy, string>;

export const misfirePolicyTranslationKeys = {
  [MISFIRE_POLICY.FIRE_ONCE]: 'misfirePolicyValues.fireOnce',
  [MISFIRE_POLICY.DO_NOTHING]: 'misfirePolicyValues.doNothing',
} as const satisfies Record<MisfirePolicy, string>;

export const jobLogStatusTranslationKeys = {
  [JOB_LOG_STATUS.SUCCESS]: 'statuses.execution.success',
  [JOB_LOG_STATUS.FAILED]: 'statuses.execution.failed',
  [JOB_LOG_STATUS.SKIPPED]: 'statuses.execution.skipped',
  [JOB_LOG_STATUS.INTERRUPTED]: 'statuses.execution.interrupted',
} as const satisfies Record<JobLogStatus, string>;

export const schedulerTriggerTranslationKeys = {
  [SCHEDULER_TRIGGER_TYPE.SCHEDULED]: 'trigger.scheduled',
  [SCHEDULER_TRIGGER_TYPE.MANUAL]: 'trigger.manual',
  [SCHEDULER_TRIGGER_TYPE.MISFIRE]: 'trigger.misfire',
} as const satisfies Record<SchedulerTriggerType, string>;

export const httpExecutionFailureTranslationKeys = {
  [HTTP_EXECUTION_FAILURE_CODE.REQUEST_BUILD]: 'executionDetail.failureValues.requestBuild',
  [HTTP_EXECUTION_FAILURE_CODE.TIMEOUT]: 'executionDetail.failureValues.timeout',
  [HTTP_EXECUTION_FAILURE_CODE.CONNECT]: 'executionDetail.failureValues.connect',
  [HTTP_EXECUTION_FAILURE_CODE.REQUEST]: 'executionDetail.failureValues.request',
  [HTTP_EXECUTION_FAILURE_CODE.RESPONSE_BODY]: 'executionDetail.failureValues.responseBody',
  [HTTP_EXECUTION_FAILURE_CODE.HTTP_STATUS]: 'executionDetail.failureValues.httpStatus',
} as const satisfies Record<HttpExecutionFailureCode, string>;
