DELETE FROM sys_job WHERE job_id = 'system-log-cleanup';

DELETE FROM sys_role_menu
WHERE menu_id IN ('114', '1140', '1141', '1142');

DELETE FROM sys_menu
WHERE menu_id IN ('114', '1140', '1141', '1142');

DELETE FROM sys_config
WHERE config_key = 'sys.observability.tracingConfig';

DROP FUNCTION ensure_system_log_partition(TIMESTAMPTZ);
DROP TABLE sys_system_log;
