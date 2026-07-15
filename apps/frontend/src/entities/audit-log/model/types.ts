export type AuditStatus = 0 | 1;
export type OperationBusinessType = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9;
export type OperationOperatorType = 0 | 1 | 2;
export type AuditSortOrder = 'asc' | 'desc';

export type OperationLogSummary = Readonly<{
  oper_id: string;
  title: string;
  business_type: OperationBusinessType;
  method: string;
  request_method: string;
  operator_type: OperationOperatorType;
  oper_name: string | null;
  dept_name: string | null;
  oper_url: string;
  oper_ip: string;
  oper_location: string;
  status: AuditStatus;
  oper_time: string;
  cost_time: number;
}>;

export type OperationLogDetail = OperationLogSummary &
  Readonly<{
    oper_param: string | null;
    json_result: string | null;
    error_msg: string | null;
  }>;

export type LoginEventType =
  | 'login_success'
  | 'login_failure'
  | 'register_success'
  | 'register_failure'
  | 'logout_success'
  | 'logout_failure'
  | 'refresh_success'
  | 'refresh_failure';

export type LoginLog = Readonly<{
  info_id: string;
  user_name: string;
  ipaddr: string;
  login_location: string;
  browser: string;
  os: string;
  status: AuditStatus;
  msg: string;
  event_type: LoginEventType;
  login_time: string;
}>;

export type OperationLogFilters = Readonly<{
  title: string;
  oper_name: string;
  oper_ip: string;
  business_type: OperationBusinessType | '';
  status: AuditStatus | '';
  begin_time: string;
  end_time: string;
}>;

export type LoginLogFilters = Readonly<{
  ipaddr: string;
  user_name: string;
  status: AuditStatus | '';
  event_type: LoginEventType | '';
  begin_time: string;
  end_time: string;
}>;

export type OperationLogFilterQuery = Readonly<{
  title?: string;
  oper_name?: string;
  oper_ip?: string;
  business_type?: OperationBusinessType;
  status?: AuditStatus;
  begin_time?: string;
  end_time?: string;
}>;

export type LoginLogFilterQuery = Readonly<{
  ipaddr?: string;
  user_name?: string;
  status?: AuditStatus;
  event_type?: LoginEventType;
  begin_time?: string;
  end_time?: string;
}>;

export type OperationLogSortField =
  'business_type' | 'oper_name' | 'status' | 'oper_time' | 'cost_time';

export type LoginLogSortField = 'user_name' | 'ipaddr' | 'status' | 'login_time';

export type OperationLogListQuery = OperationLogFilterQuery &
  Readonly<{ sort_by?: OperationLogSortField; sort_order?: AuditSortOrder }>;

export type LoginLogListQuery = LoginLogFilterQuery &
  Readonly<{ sort_by?: LoginLogSortField; sort_order?: AuditSortOrder }>;
