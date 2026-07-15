DELETE FROM sys_role_menu
WHERE menu_id IN ('111', '112', '113', '1120', '1121', '1122', '1130', '1131', '1132');

DELETE FROM sys_menu
WHERE menu_id IN ('111', '112', '113', '1120', '1121', '1122', '1130', '1131', '1132');

DELETE FROM sys_dict_data WHERE dict_type = 'sys_oper_type';
DELETE FROM sys_dict_type WHERE dict_type = 'sys_oper_type';
DELETE FROM sys_config WHERE config_key = 'sys.auth.loginLockConfig';

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.auth.ipLocationConfig') THEN
        RAISE EXCEPTION 'cannot restore sys.auth.ipLocationConfig: key already exists';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.client.ipLocationConfig') THEN
        RAISE EXCEPTION 'cannot restore missing sys.client.ipLocationConfig';
    END IF;
END $$;

UPDATE sys_config
SET config_name = '认证-IP 归属地配置',
    config_key = 'sys.auth.ipLocationConfig',
    update_by = 'admin',
    update_time = CURRENT_TIMESTAMP,
    remark = 'IP 归属地解析配置 JSON。enabled 控制是否调用 pconline 解析公网 IP；关闭时登录地点显示 XX XX，内网 IP 始终显示内网IP。'
WHERE config_key = 'sys.client.ipLocationConfig';

DROP TABLE audit_outbox;
DROP TABLE sys_logininfor;
DROP TABLE sys_oper_log;
