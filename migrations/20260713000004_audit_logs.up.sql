CREATE TABLE sys_oper_log (
    oper_id VARCHAR(36) PRIMARY KEY,
    request_id VARCHAR(64) NOT NULL DEFAULT '',
    title VARCHAR(50) NOT NULL,
    business_type SMALLINT NOT NULL,
    method VARCHAR(255) NOT NULL,
    request_method VARCHAR(10) NOT NULL,
    operator_type SMALLINT NOT NULL DEFAULT 1,
    operator_id VARCHAR(36) NULL,
    oper_name VARCHAR(50) NOT NULL DEFAULT '',
    dept_id VARCHAR(36) NULL,
    dept_name VARCHAR(50) NOT NULL DEFAULT '',
    oper_url VARCHAR(255) NOT NULL DEFAULT '',
    oper_ip VARCHAR(45) NOT NULL DEFAULT '',
    oper_location_kind VARCHAR(16) NOT NULL,
    oper_location VARCHAR(255) NOT NULL DEFAULT '',
    oper_param VARCHAR(2000) NOT NULL DEFAULT '',
    json_result VARCHAR(2000) NOT NULL DEFAULT '',
    status SMALLINT NOT NULL,
    error_msg VARCHAR(2000) NOT NULL DEFAULT '',
    oper_time TIMESTAMPTZ NOT NULL,
    cost_time BIGINT NOT NULL,
    CONSTRAINT chk_sys_oper_log_business_type CHECK (business_type BETWEEN 0 AND 9),
    CONSTRAINT chk_sys_oper_log_operator_type CHECK (operator_type BETWEEN 0 AND 2),
    CONSTRAINT chk_sys_oper_log_status CHECK (status IN (0, 1)),
    CONSTRAINT chk_sys_oper_log_cost_time CHECK (cost_time >= 0),
    CONSTRAINT chk_sys_oper_log_location_kind CHECK (oper_location_kind IN ('resolved', 'internal', 'unknown')),
    CONSTRAINT chk_sys_oper_log_location_value CHECK (
        (oper_location_kind = 'resolved' AND BTRIM(oper_location) <> '')
        OR (oper_location_kind IN ('internal', 'unknown') AND oper_location = '')
    )
);

CREATE INDEX idx_sys_oper_log_business_type ON sys_oper_log (business_type);
CREATE INDEX idx_sys_oper_log_status ON sys_oper_log (status);
CREATE INDEX idx_sys_oper_log_time ON sys_oper_log (oper_time DESC, oper_id DESC);

CREATE TABLE audit_outbox (
    outbox_id VARCHAR(36) PRIMARY KEY,
    stream VARCHAR(16) NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    payload_version SMALLINT NOT NULL,
    payload JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    available_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    attempt_count INTEGER NOT NULL DEFAULT 0,
    lease_token VARCHAR(36) NULL,
    lease_until TIMESTAMPTZ NULL,
    processed_at TIMESTAMPTZ NULL,
    last_error_code VARCHAR(96) NULL,
    CONSTRAINT chk_audit_outbox_stream CHECK (stream IN ('operation', 'security')),
    CONSTRAINT chk_audit_outbox_event_type CHECK (
        event_type IN (
            'operation',
            'login_success', 'login_failure',
            'register_success', 'register_failure',
            'logout_success', 'logout_failure',
            'refresh_success', 'refresh_failure'
        )
    ),
    CONSTRAINT chk_audit_outbox_stream_event_type CHECK (
        (stream = 'operation' AND event_type = 'operation')
        OR (
            stream = 'security'
            AND event_type IN (
                'login_success', 'login_failure',
                'register_success', 'register_failure',
                'logout_success', 'logout_failure',
                'refresh_success', 'refresh_failure'
            )
        )
    ),
    CONSTRAINT chk_audit_outbox_payload_version CHECK (payload_version = 1),
    CONSTRAINT chk_audit_outbox_attempt_count CHECK (attempt_count >= 0),
    CONSTRAINT chk_audit_outbox_lease CHECK (
        (lease_token IS NULL AND lease_until IS NULL)
        OR (lease_token IS NOT NULL AND lease_until IS NOT NULL)
    ),
    CONSTRAINT chk_audit_outbox_processed_lease CHECK (
        processed_at IS NULL OR (lease_token IS NULL AND lease_until IS NULL)
    )
);

CREATE INDEX idx_audit_outbox_ready ON audit_outbox (available_at ASC, occurred_at ASC, outbox_id ASC)
WHERE processed_at IS NULL;
CREATE INDEX idx_audit_outbox_processed ON audit_outbox (processed_at ASC)
WHERE processed_at IS NOT NULL;

CREATE TABLE sys_logininfor (
    info_id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NULL,
    user_name VARCHAR(50) NOT NULL DEFAULT '',
    ipaddr VARCHAR(45) NOT NULL DEFAULT '',
    login_location_kind VARCHAR(16) NOT NULL,
    login_location VARCHAR(255) NOT NULL DEFAULT '',
    browser VARCHAR(50) NOT NULL DEFAULT '',
    os VARCHAR(50) NOT NULL DEFAULT '',
    status SMALLINT NOT NULL,
    event_type VARCHAR(32) NOT NULL,
    message_key VARCHAR(160) NOT NULL,
    message_params JSONB NOT NULL DEFAULT '{}'::JSONB,
    login_time TIMESTAMPTZ NOT NULL,
    CONSTRAINT chk_sys_logininfor_status CHECK (status IN (0, 1)),
    CONSTRAINT chk_sys_logininfor_event_type CHECK (
        event_type IN (
            'login_success', 'login_failure',
            'register_success', 'register_failure',
            'logout_success', 'logout_failure',
            'refresh_success', 'refresh_failure'
        )
    ),
    CONSTRAINT chk_sys_logininfor_location_kind CHECK (login_location_kind IN ('resolved', 'internal', 'unknown')),
    CONSTRAINT chk_sys_logininfor_location_value CHECK (
        (login_location_kind = 'resolved' AND BTRIM(login_location) <> '')
        OR (login_location_kind IN ('internal', 'unknown') AND login_location = '')
    )
);

CREATE INDEX idx_sys_logininfor_status ON sys_logininfor (status);
CREATE INDEX idx_sys_logininfor_event_type ON sys_logininfor (event_type);
CREATE INDEX idx_sys_logininfor_time ON sys_logininfor (login_time DESC, info_id DESC);

INSERT INTO sys_dict_type (dict_id, dict_name, dict_type, status, create_by, create_time, remark)
VALUES ('audit-oper-type', '操作类型', 'sys_oper_type', '0', 'admin', CURRENT_TIMESTAMP, '操作日志业务类型');

INSERT INTO sys_dict_data (
    dict_code, dict_sort, dict_label, dict_value, dict_type, css_class, list_class,
    is_default, status, create_by, create_time, remark
)
VALUES
    ('audit-oper-type-other', 0, '其他', '0', 'sys_oper_type', '', 'info', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '其他操作'),
    ('audit-oper-type-insert', 1, '新增', '1', 'sys_oper_type', '', 'success', 'N', '0', 'admin', CURRENT_TIMESTAMP, '新增数据'),
    ('audit-oper-type-update', 2, '修改', '2', 'sys_oper_type', '', 'primary', 'N', '0', 'admin', CURRENT_TIMESTAMP, '修改数据'),
    ('audit-oper-type-delete', 3, '删除', '3', 'sys_oper_type', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '删除数据'),
    ('audit-oper-type-grant', 4, '授权', '4', 'sys_oper_type', '', 'warning', 'N', '0', 'admin', CURRENT_TIMESTAMP, '授权操作'),
    ('audit-oper-type-export', 5, '导出', '5', 'sys_oper_type', '', 'info', 'N', '0', 'admin', CURRENT_TIMESTAMP, '导出数据'),
    ('audit-oper-type-import', 6, '导入', '6', 'sys_oper_type', '', 'info', 'N', '0', 'admin', CURRENT_TIMESTAMP, '导入数据'),
    ('audit-oper-type-force', 7, '强退', '7', 'sys_oper_type', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '强制退出'),
    ('audit-oper-type-gencode', 8, '生成代码', '8', 'sys_oper_type', '', 'primary', 'N', '0', 'admin', CURRENT_TIMESTAMP, '生成代码'),
    ('audit-oper-type-clean', 9, '清空', '9', 'sys_oper_type', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '清空数据');

INSERT INTO sys_menu (
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark
)
VALUES
    ('111', '日志管理', '3', 4, '/dashboard/monitor/logs', NULL, '', '', FALSE, FALSE, 'M', '0', '0', NULL, 'icon.logs', 'admin', CURRENT_TIMESTAMP, '日志管理目录'),
    ('112', '操作日志', '111', 1, '/dashboard/monitor/logs/operation-logs', 'monitor/logs/operation/index', '', 'OperationLogs', FALSE, FALSE, 'C', '0', '0', 'system:operlog:list', 'icon.operation-log', 'admin', CURRENT_TIMESTAMP, '操作日志菜单'),
    ('113', '登录日志', '111', 2, '/dashboard/monitor/logs/login-logs', 'monitor/logs/login/index', '', 'LoginLogs', FALSE, FALSE, 'C', '0', '0', 'system:logininfor:list', 'icon.login-log', 'admin', CURRENT_TIMESTAMP, '登录日志菜单'),
    ('1120', '操作日志详情', '112', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:operlog:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1121', '操作日志删除', '112', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:operlog:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1122', '操作日志导出', '112', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:operlog:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1130', '登录日志删除', '113', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:logininfor:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1131', '登录日志导出', '113', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:logininfor:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1132', '账户解锁', '113', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:logininfor:unlock', '#', 'admin', CURRENT_TIMESTAMP, '清除账户密码失败计数');

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 'admin', menu_id
FROM sys_menu
WHERE menu_id IN ('111', '112', '113', '1120', '1121', '1122', '1130', '1131', '1132');

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.client.ipLocationConfig') THEN
        RAISE EXCEPTION 'cannot migrate sys.auth.ipLocationConfig: sys.client.ipLocationConfig already exists';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM sys_config WHERE config_key = 'sys.auth.ipLocationConfig') THEN
        RAISE EXCEPTION 'cannot migrate missing sys.auth.ipLocationConfig';
    END IF;
END $$;

UPDATE sys_config
SET config_name = '客户端-IP 归属地配置',
    config_key = 'sys.client.ipLocationConfig',
    update_by = 'admin',
    update_time = CURRENT_TIMESTAMP,
    remark = '客户端 IP 归属地解析配置 JSON。enabled 控制是否调用 pconline 解析公网 IP；关闭时地点显示本地化未知文案，内网 IP 显示本地化内网文案；在线会话与审计日志共用此配置。'
WHERE config_key = 'sys.auth.ipLocationConfig';

INSERT INTO sys_config (
    config_id, config_name, config_key, config_value, config_type, public_read,
    create_by, create_time, remark
)
VALUES (
    '16', '认证-登录锁定配置', 'sys.auth.loginLockConfig',
    '{"max_retry_count":5,"lock_minutes":10}', 'Y', FALSE, 'admin', CURRENT_TIMESTAMP,
    '登录密码错误锁定配置 JSON。max_retry_count 是连续错误次数阈值，lock_minutes 是达到阈值后的锁定分钟数；仅后端读取。'
);
