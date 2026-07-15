import type {
  AuditStatus,
  LoginEventType,
  OperationBusinessType,
  OperationOperatorType,
} from './types';

export const AUDIT_STATUS = { SUCCESS: 0, FAILURE: 1 } as const;

export const OPERATION_BUSINESS_TYPE = {
  OTHER: 0,
  INSERT: 1,
  UPDATE: 2,
  DELETE: 3,
  GRANT: 4,
  EXPORT: 5,
  IMPORT: 6,
  FORCE: 7,
  GENERATE: 8,
  CLEAN: 9,
} as const;

export const operationBusinessTypeKeys: Record<OperationBusinessType, string> = {
  [OPERATION_BUSINESS_TYPE.OTHER]: 'businessTypes.other',
  [OPERATION_BUSINESS_TYPE.INSERT]: 'businessTypes.insert',
  [OPERATION_BUSINESS_TYPE.UPDATE]: 'businessTypes.update',
  [OPERATION_BUSINESS_TYPE.DELETE]: 'businessTypes.delete',
  [OPERATION_BUSINESS_TYPE.GRANT]: 'businessTypes.grant',
  [OPERATION_BUSINESS_TYPE.EXPORT]: 'businessTypes.export',
  [OPERATION_BUSINESS_TYPE.IMPORT]: 'businessTypes.import',
  [OPERATION_BUSINESS_TYPE.FORCE]: 'businessTypes.force',
  [OPERATION_BUSINESS_TYPE.GENERATE]: 'businessTypes.generate',
  [OPERATION_BUSINESS_TYPE.CLEAN]: 'businessTypes.clean',
};

export const auditStatusKeys: Record<AuditStatus, string> = {
  [AUDIT_STATUS.SUCCESS]: 'statuses.success',
  [AUDIT_STATUS.FAILURE]: 'statuses.failure',
};

export const operationOperatorTypeKeys: Record<OperationOperatorType, string> = {
  0: 'operatorTypes.other',
  1: 'operatorTypes.backendUser',
  2: 'operatorTypes.mobileUser',
};

export const LOGIN_EVENT_TYPES: readonly LoginEventType[] = [
  'login_success',
  'login_failure',
  'register_success',
  'register_failure',
  'logout_success',
  'logout_failure',
  'refresh_success',
  'refresh_failure',
];

export const loginEventTypeKeys: Record<LoginEventType, string> = {
  login_success: 'eventTypes.loginSuccess',
  login_failure: 'eventTypes.loginFailure',
  register_success: 'eventTypes.registerSuccess',
  register_failure: 'eventTypes.registerFailure',
  logout_success: 'eventTypes.logoutSuccess',
  logout_failure: 'eventTypes.logoutFailure',
  refresh_success: 'eventTypes.refreshSuccess',
  refresh_failure: 'eventTypes.refreshFailure',
};
