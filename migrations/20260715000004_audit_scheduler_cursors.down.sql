DROP INDEX idx_sys_job_execution_group_cursor;
DROP INDEX idx_sys_job_execution_trigger_cursor;
DROP INDEX idx_sys_job_execution_outcome_time;
DROP INDEX idx_sys_job_execution_job_time;
DROP INDEX idx_sys_job_execution_terminal_cursor;
DROP INDEX idx_sys_job_group_status;
DROP INDEX idx_sys_job_created_cursor;

CREATE INDEX idx_sys_job_group_status ON sys_job (job_group, status);
CREATE INDEX idx_sys_job_execution_job_time ON sys_job_execution (job_id, create_time DESC);
CREATE INDEX idx_sys_job_execution_outcome_time
    ON sys_job_execution (outcome, create_time DESC)
    WHERE state = 'T';

DROP INDEX idx_sys_logininfor_ingested_cursor;
DROP INDEX idx_sys_logininfor_status_cursor;
DROP INDEX idx_sys_logininfor_ip_cursor;
DROP INDEX idx_sys_logininfor_user_cursor;
DROP INDEX idx_sys_oper_log_ingested_cursor;
DROP INDEX idx_sys_oper_log_cost_cursor;
DROP INDEX idx_sys_oper_log_operator_cursor;
DROP INDEX idx_sys_oper_log_status_cursor;
DROP INDEX idx_sys_oper_log_business_type_cursor;

CREATE INDEX idx_sys_oper_log_business_type ON sys_oper_log (business_type);
CREATE INDEX idx_sys_oper_log_status ON sys_oper_log (status);
CREATE INDEX idx_sys_logininfor_status ON sys_logininfor (status);

ALTER TABLE sys_logininfor DROP COLUMN ingested_at;
ALTER TABLE sys_oper_log DROP COLUMN ingested_at;
