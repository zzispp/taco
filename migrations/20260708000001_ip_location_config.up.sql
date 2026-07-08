INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, public_read, create_by, create_time, remark)
SELECT '15', '认证-IP 归属地配置', 'sys.auth.ipLocationConfig', '{"enabled":false}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
       'IP 归属地解析配置 JSON。enabled 控制是否调用 pconline 解析公网 IP；关闭时登录地点显示 XX XX，内网 IP 始终显示内网IP。'
WHERE NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.auth.ipLocationConfig');
