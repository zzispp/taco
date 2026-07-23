DROP FUNCTION drop_expired_system_log_partition(TEXT, TIMESTAMPTZ);

CREATE INDEX idx_sys_system_log_cursor
    ON sys_system_log (occurred_at DESC, id DESC);

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

UPDATE sys_job
SET remark = '按滚动保留周期分批删除过期系统日志。retention_days 控制保留天数，batch_size 控制每个独立删除事务的最大记录数；任务默认启用且不可停用或删除。'
WHERE job_id = 'system-log-cleanup';
