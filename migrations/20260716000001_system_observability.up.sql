CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE TABLE sys_system_log (
    id VARCHAR(36) NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    level VARCHAR(5) NOT NULL,
    target VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    fields JSONB NOT NULL DEFAULT '{}'::jsonb,
    searchable_content TEXT GENERATED ALWAYS AS (message || ' ' || fields::text) STORED,
    search_document TSVECTOR GENERATED ALWAYS AS (to_tsvector('simple', message || ' ' || fields::text)) STORED,
    CONSTRAINT pk_sys_system_log PRIMARY KEY (occurred_at, id),
    CONSTRAINT chk_sys_system_log_level CHECK (level IN ('trace', 'debug', 'info', 'warn', 'error')),
    CONSTRAINT chk_sys_system_log_target CHECK (BTRIM(target) <> ''),
    CONSTRAINT chk_sys_system_log_fields_object CHECK (jsonb_typeof(fields) = 'object')
) PARTITION BY RANGE (occurred_at);

CREATE INDEX idx_sys_system_log_cursor ON sys_system_log (occurred_at DESC, id DESC);
CREATE INDEX idx_sys_system_log_target_cursor ON sys_system_log (target, occurred_at DESC, id DESC);
CREATE INDEX idx_sys_system_log_level_cursor ON sys_system_log (level, occurred_at DESC, id DESC);
CREATE INDEX idx_sys_system_log_search_document ON sys_system_log USING GIN (search_document);
CREATE INDEX idx_sys_system_log_search_content_trgm ON sys_system_log USING GIN (searchable_content gin_trgm_ops);

CREATE OR REPLACE FUNCTION ensure_system_log_partition(value_occurred_at TIMESTAMPTZ)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
    partition_start TIMESTAMPTZ := date_trunc('day', value_occurred_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC';
    partition_end TIMESTAMPTZ := partition_start + INTERVAL '1 day';
    partition_name TEXT := format('sys_system_log_%s', to_char(partition_start AT TIME ZONE 'UTC', 'YYYYMMDD'));
BEGIN
    IF to_regclass(partition_name) IS NOT NULL THEN
        RETURN;
    END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended(partition_name, 0));
    IF to_regclass(partition_name) IS NOT NULL THEN
        RETURN;
    END IF;
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF sys_system_log FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        partition_start,
        partition_end
    );
END;
$$;

INSERT INTO sys_config (
    config_id, config_name, config_key, config_value, config_type, public_read,
    create_by, create_time, remark
)
VALUES (
    '17',
    '可观测性-日志配置',
    'sys.observability.tracingConfig',
    '{"log_level":"info","http":{"access_enabled":true,"capture_request_body":false,"capture_response_body":false,"capture_query_parameters":true,"capture_request_headers":false,"max_body_capture_bytes":16384},"slow_operation_ms":{"postgres":500,"redis":100,"outbound_http":1000}}',
    'Y',
    FALSE,
    'admin',
    CURRENT_TIMESTAMP,
    '系统日志运行配置 JSON。log_level 控制持久化日志级别；http 配置访问日志和请求体、响应体、查询参数、请求头采集及 max_body_capture_bytes 上限；slow_operation_ms 配置 PostgreSQL、Redis、出站 HTTP 慢调用阈值。参数更新通过 PostgreSQL 通知在线热重载。'
);

INSERT INTO sys_menu (
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark
)
VALUES
    ('114', '系统日志', '111', 3, '/dashboard/monitor/logs/system-logs', 'monitor/logs/system/index', '', 'SystemLogs', FALSE, FALSE, 'C', '0', '0', 'system:systemlog:list', 'icon.system-log', 'admin', CURRENT_TIMESTAMP, '系统运行日志菜单'),
    ('1140', '系统日志详情', '114', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:systemlog:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1141', '系统日志删除', '114', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:systemlog:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1142', '系统日志导出', '114', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:systemlog:export', '#', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT '2', menu_id
FROM sys_menu
WHERE menu_id IN ('114', '1140', '1141', '1142');

INSERT INTO sys_job (
    job_id, job_name, job_group, task_key, task_params, params_schema_version,
    repeatable, invoke_target, cron_expression, misfire_policy, concurrent,
    status, create_by, create_time, remark
)
VALUES (
    'system-log-cleanup',
    '系统日志清理',
    'SYSTEM',
    'observability.cleanupSystemLogs',
    '{"retention_days":7,"batch_size":1000}'::jsonb,
    1,
    FALSE,
    'observability.cleanupSystemLogs(retention_days=7,batch_size=1000)',
    '0 0 19 * * *',
    '2',
    '1',
    '0',
    'admin',
    CURRENT_TIMESTAMP,
    '按滚动保留周期分批删除过期系统日志。retention_days 控制保留天数，batch_size 控制每个独立删除事务的最大记录数；任务默认启用且不可停用或删除。'
);
