ALTER TABLE sys_oper_log
    ADD COLUMN ingested_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp();

ALTER TABLE sys_logininfor
    ADD COLUMN ingested_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp();

DROP INDEX idx_sys_oper_log_business_type;
DROP INDEX idx_sys_oper_log_status;
DROP INDEX idx_sys_logininfor_status;

CREATE INDEX idx_sys_oper_log_business_type_cursor
    ON sys_oper_log (business_type, oper_id);
CREATE INDEX idx_sys_oper_log_status_cursor
    ON sys_oper_log (status, oper_id);
CREATE INDEX idx_sys_oper_log_operator_cursor
    ON sys_oper_log (oper_name, oper_id);
CREATE INDEX idx_sys_oper_log_cost_cursor
    ON sys_oper_log (cost_time, oper_id);
CREATE INDEX idx_sys_oper_log_ingested_cursor
    ON sys_oper_log (ingested_at DESC, oper_id DESC);

CREATE INDEX idx_sys_logininfor_user_cursor
    ON sys_logininfor (user_name, info_id);
CREATE INDEX idx_sys_logininfor_ip_cursor
    ON sys_logininfor (ipaddr, info_id);
CREATE INDEX idx_sys_logininfor_status_cursor
    ON sys_logininfor (status, info_id);
CREATE INDEX idx_sys_logininfor_ingested_cursor
    ON sys_logininfor (ingested_at DESC, info_id DESC);

DROP INDEX idx_sys_job_group_status;
DROP INDEX idx_sys_job_execution_job_time;
DROP INDEX idx_sys_job_execution_outcome_time;

CREATE INDEX idx_sys_job_created_cursor
    ON sys_job (create_time DESC, job_id DESC);
CREATE INDEX idx_sys_job_group_status
    ON sys_job (job_group, status, create_time DESC, job_id DESC);

CREATE INDEX idx_sys_job_execution_terminal_cursor
    ON sys_job_execution (create_time DESC, execution_id DESC)
    WHERE state = 'T';
CREATE INDEX idx_sys_job_execution_job_time
    ON sys_job_execution (job_id, create_time DESC, execution_id DESC);
CREATE INDEX idx_sys_job_execution_outcome_time
    ON sys_job_execution (outcome, create_time DESC, execution_id DESC)
    WHERE state = 'T';
CREATE INDEX idx_sys_job_execution_trigger_cursor
    ON sys_job_execution (trigger_type, create_time DESC, execution_id DESC)
    WHERE state = 'T';
CREATE INDEX idx_sys_job_execution_group_cursor
    ON sys_job_execution (job_group, create_time DESC, execution_id DESC)
    WHERE state = 'T';
