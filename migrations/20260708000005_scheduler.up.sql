CREATE TABLE sys_job (
    job_id VARCHAR(36) PRIMARY KEY,
    job_name VARCHAR(64) NOT NULL,
    job_group VARCHAR(64) NOT NULL,
    task_key VARCHAR(128) NOT NULL,
    task_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    params_schema_version SMALLINT NOT NULL DEFAULT 1,
    repeatable BOOLEAN NOT NULL DEFAULT FALSE,
    invoke_target VARCHAR(500) NOT NULL,
    cron_expression VARCHAR(255) NOT NULL,
    misfire_policy CHAR(1) NOT NULL DEFAULT '3',
    concurrent CHAR(1) NOT NULL DEFAULT '1',
    status CHAR(1) NOT NULL DEFAULT '1',
    schedule_revision BIGINT NOT NULL DEFAULT 1,
    next_run_at TIMESTAMPTZ NULL,
    runtime_error_code VARCHAR(128) NULL,
    runtime_error_time TIMESTAMPTZ NULL,
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL,
    CONSTRAINT chk_sys_job_task_params_object CHECK (jsonb_typeof(task_params) = 'object'),
    CONSTRAINT chk_sys_job_schema_version CHECK (params_schema_version > 0),
    CONSTRAINT chk_sys_job_misfire_policy CHECK (misfire_policy IN ('2', '3')),
    CONSTRAINT chk_sys_job_concurrent CHECK (concurrent IN ('0', '1')),
    CONSTRAINT chk_sys_job_status CHECK (status IN ('0', '1')),
    CONSTRAINT chk_sys_job_schedule_revision CHECK (schedule_revision > 0),
    CONSTRAINT chk_sys_job_runtime_error CHECK (
        (runtime_error_code IS NULL AND runtime_error_time IS NULL)
        OR (runtime_error_code IS NOT NULL AND runtime_error_time IS NOT NULL)
    )
);

CREATE TABLE sys_job_execution (
    execution_id VARCHAR(36) PRIMARY KEY,
    job_id VARCHAR(36) NOT NULL,
    job_revision BIGINT NOT NULL,
    job_name VARCHAR(64) NOT NULL,
    job_group VARCHAR(64) NOT NULL,
    task_key VARCHAR(128) NOT NULL,
    task_params JSONB NOT NULL,
    params_schema_version SMALLINT NOT NULL,
    repeatable BOOLEAN NOT NULL,
    invoke_target VARCHAR(500) NOT NULL,
    concurrent CHAR(1) NOT NULL,
    trigger_type CHAR(1) NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    state CHAR(1) NOT NULL,
    outcome CHAR(1) NULL,
    executor_epoch VARCHAR(64) NULL,
    requested_by VARCHAR(64) NULL,
    message_key VARCHAR(255) NULL,
    message_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_key VARCHAR(255) NULL,
    error_params JSONB NOT NULL DEFAULT '{}'::jsonb,
    start_time TIMESTAMPTZ NULL,
    end_time TIMESTAMPTZ NULL,
    create_time TIMESTAMPTZ NOT NULL,
    CONSTRAINT chk_sys_job_execution_revision CHECK (job_revision > 0),
    CONSTRAINT chk_sys_job_execution_task_params CHECK (jsonb_typeof(task_params) = 'object'),
    CONSTRAINT chk_sys_job_execution_schema_version CHECK (params_schema_version > 0),
    CONSTRAINT chk_sys_job_execution_concurrent CHECK (concurrent IN ('0', '1')),
    CONSTRAINT chk_sys_job_execution_trigger CHECK (trigger_type IN ('S', 'F', 'M')),
    CONSTRAINT chk_sys_job_execution_state CHECK (state IN ('P', 'R', 'T')),
    CONSTRAINT chk_sys_job_execution_outcome CHECK (outcome IS NULL OR outcome IN ('0', '1', '2', '3')),
    CONSTRAINT chk_sys_job_execution_message_params CHECK (jsonb_typeof(message_params) = 'object'),
    CONSTRAINT chk_sys_job_execution_error_params CHECK (jsonb_typeof(error_params) = 'object'),
    CONSTRAINT chk_sys_job_execution_localized_payload CHECK (
        (
            state IN ('P', 'R')
            AND message_key IS NULL
            AND message_params = '{}'::jsonb
            AND error_key IS NULL
            AND error_params = '{}'::jsonb
        )
        OR (
            state = 'T'
            AND message_key IS NOT NULL
            AND (error_key IS NOT NULL OR error_params = '{}'::jsonb)
        )
    ),
    CONSTRAINT chk_sys_job_execution_requester CHECK (
        (trigger_type = 'M' AND requested_by IS NOT NULL)
        OR (trigger_type <> 'M' AND requested_by IS NULL)
    ),
    CONSTRAINT chk_sys_job_execution_lifecycle CHECK (
        (state = 'P' AND outcome IS NULL AND executor_epoch IS NULL AND start_time IS NULL AND end_time IS NULL)
        OR (state = 'R' AND outcome IS NULL AND executor_epoch IS NOT NULL AND start_time IS NOT NULL AND end_time IS NULL)
        OR (state = 'T' AND outcome IS NOT NULL AND end_time IS NOT NULL AND (outcome = '2' OR start_time IS NOT NULL))
    ),
    CONSTRAINT chk_sys_job_execution_time_order CHECK (end_time IS NULL OR start_time IS NULL OR end_time >= start_time)
);

CREATE UNIQUE INDEX idx_sys_job_task_key_singleton ON sys_job (task_key) WHERE repeatable = FALSE;
CREATE INDEX idx_sys_job_group_status ON sys_job (job_group, status);
CREATE INDEX idx_sys_job_due ON sys_job (next_run_at) WHERE status = '0';
CREATE UNIQUE INDEX idx_sys_job_execution_occurrence
    ON sys_job_execution (job_id, job_revision, scheduled_at)
    WHERE trigger_type IN ('S', 'F');
CREATE UNIQUE INDEX idx_sys_job_execution_active_disallow
    ON sys_job_execution (job_id)
    WHERE concurrent = '1' AND state IN ('P', 'R');
CREATE INDEX idx_sys_job_execution_dispatch ON sys_job_execution (state, scheduled_at, create_time);
CREATE INDEX idx_sys_job_execution_job_time ON sys_job_execution (job_id, create_time DESC);
CREATE INDEX idx_sys_job_execution_outcome_time ON sys_job_execution (outcome, create_time DESC) WHERE state = 'T';

INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark)
VALUES
    ('108', '定时任务', '1', 9, '/dashboard/admin/jobs', 'system/job/index', '', 'Job', FALSE, FALSE, 'C', '0', '0', 'system:job:list', 'icon.job', 'admin', CURRENT_TIMESTAMP, '定时任务菜单'),
    ('109', '调度日志', '1', 10, '/dashboard/admin/job-logs', 'system/job/log', '', 'JobLog', FALSE, FALSE, 'C', '0', '0', 'system:job:log:list', 'icon.job-log', 'admin', CURRENT_TIMESTAMP, '调度日志菜单'),
    ('1080', '定时任务查询', '108', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1081', '定时任务导入', '108', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:import', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1082', '定时任务修改', '108', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1083', '定时任务删除', '108', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1084', '定时任务导出', '108', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1085', '定时任务状态', '108', 6, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:changeStatus', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1086', '定时任务执行', '108', 7, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:run', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1090', '调度日志查询', '109', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:log:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1091', '调度日志删除', '109', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:log:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1092', '调度日志导出', '109', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:job:log:export', '#', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_dict_type (dict_id, dict_name, dict_type, status, create_by, create_time, remark)
VALUES
    ('scheduler-job-group', '任务分组', 'sys_job_group', '0', 'admin', CURRENT_TIMESTAMP, '定时任务分组'),
    ('scheduler-job-status', '任务状态', 'sys_job_status', '0', 'admin', CURRENT_TIMESTAMP, '定时任务状态');

INSERT INTO sys_dict_data (dict_code, dict_sort, dict_label, dict_value, dict_type, css_class, list_class, is_default, status, create_by, create_time, remark)
VALUES
    ('scheduler-job-group-default', 1, '默认', 'DEFAULT', 'sys_job_group', '', 'primary', 'N', '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('scheduler-job-group-system', 2, '系统', 'SYSTEM', 'sys_job_group', '', 'success', 'Y', '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('scheduler-job-status-normal', 1, '正常', '0', 'sys_job_status', '', 'primary', 'Y', '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('scheduler-job-status-paused', 2, '暂停', '1', 'sys_job_status', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '');
