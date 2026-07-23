DROP INDEX IF EXISTS idx_sys_system_log_cursor;

CREATE OR REPLACE FUNCTION ensure_system_log_partition(value_occurred_at TIMESTAMPTZ)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
    partition_start TIMESTAMPTZ := date_trunc('day', value_occurred_at AT TIME ZONE 'UTC') AT TIME ZONE 'UTC';
    partition_end TIMESTAMPTZ := partition_start + INTERVAL '1 day';
    partition_name TEXT := format('sys_system_log_%s', to_char(partition_start AT TIME ZONE 'UTC', 'YYYYMMDD'));
BEGIN
    PERFORM pg_advisory_xact_lock(hashtextextended(partition_name, 0));
    IF to_regclass(format('public.%I', partition_name)) IS NOT NULL THEN
        RETURN;
    END IF;
    EXECUTE format(
        'CREATE TABLE public.%I PARTITION OF public.sys_system_log FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        partition_start,
        partition_end
    );
END;
$$;

CREATE OR REPLACE FUNCTION drop_expired_system_log_partition(value_partition_name TEXT, value_cutoff TIMESTAMPTZ)
RETURNS BIGINT
LANGUAGE plpgsql
AS $$
DECLARE
    partition_suffix TEXT := right(value_partition_name, 8);
    partition_date DATE;
    partition_end TIMESTAMPTZ;
    partition_attached BOOLEAN;
    deleted_rows BIGINT;
BEGIN
    IF value_partition_name !~ '^sys_system_log_[0-9]{8}$' THEN
        RAISE EXCEPTION 'invalid system log partition name: %', value_partition_name;
    END IF;
    partition_date := to_date(partition_suffix, 'YYYYMMDD');
    IF to_char(partition_date, 'YYYYMMDD') <> partition_suffix THEN
        RAISE EXCEPTION 'invalid system log partition date: %', value_partition_name;
    END IF;
    partition_end := (partition_date::TIMESTAMP AT TIME ZONE 'UTC') + INTERVAL '1 day';
    IF partition_end > value_cutoff THEN
        RAISE EXCEPTION 'system log partition % is not fully expired', value_partition_name;
    END IF;

    PERFORM pg_advisory_xact_lock(hashtextextended(value_partition_name, 0));
    SELECT EXISTS(
        SELECT 1
        FROM pg_inherits inheritance
        JOIN pg_class child ON child.oid = inheritance.inhrelid
        JOIN pg_namespace child_namespace ON child_namespace.oid = child.relnamespace
        WHERE inheritance.inhparent = 'public.sys_system_log'::regclass
          AND child_namespace.nspname = 'public'
          AND child.relname = value_partition_name
    ) INTO partition_attached;
    IF NOT partition_attached THEN
        RETURN NULL;
    END IF;

    EXECUTE format('LOCK TABLE public.%I IN ACCESS EXCLUSIVE MODE', value_partition_name);
    EXECUTE format('SELECT COUNT(*) FROM public.%I', value_partition_name) INTO deleted_rows;
    EXECUTE format('DROP TABLE public.%I', value_partition_name);
    RETURN deleted_rows;
END;
$$;

UPDATE sys_job
SET remark = '按滚动保留周期清理过期系统日志。完整过期的 UTC 日分区按独立事务删除；截止日分区按行精确清理，batch_size 仅控制截止日单个删除事务的最大记录数；任务默认启用且不可停用或删除。'
WHERE job_id = 'system-log-cleanup';
