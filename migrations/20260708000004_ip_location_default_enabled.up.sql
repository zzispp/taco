UPDATE sys_config
SET config_value = '{"enabled":true}', update_by = 'admin', update_time = CURRENT_TIMESTAMP
WHERE config_key = 'sys.auth.ipLocationConfig';
