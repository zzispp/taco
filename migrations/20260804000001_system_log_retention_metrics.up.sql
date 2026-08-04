CREATE OR REPLACE FUNCTION drop_expired_system_log_partition(value_partition_name TEXT, value_cutoff TIMESTAMPTZ)
RETURNS BIGINT
LANGUAGE plpgsql
AS $$
DECLARE
    partition_suffix TEXT := right(value_partition_name, 8);
    partition_date DATE;
    partition_end TIMESTAMPTZ;
    partition_attached BOOLEAN;
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

    EXECUTE format('DROP TABLE public.%I', value_partition_name);
    RETURN 1;
END;
$$;
