CREATE TABLE sys_notice (
    notice_id VARCHAR(36) PRIMARY KEY,
    notice_title VARCHAR(50) NOT NULL,
    notice_type CHAR(1) NOT NULL,
    notice_content TEXT NOT NULL DEFAULT '',
    status CHAR(1) NOT NULL DEFAULT '0',
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL,
    CONSTRAINT chk_sys_notice_type CHECK (notice_type IN ('1', '2')),
    CONSTRAINT chk_sys_notice_status CHECK (status IN ('0', '1'))
);

CREATE INDEX idx_sys_notice_order ON sys_notice (create_time DESC, notice_id DESC);
CREATE INDEX idx_sys_notice_active ON sys_notice (status, create_time DESC, notice_id DESC);

CREATE TABLE sys_notice_read (
    read_id VARCHAR(36) PRIMARY KEY,
    notice_id VARCHAR(36) NOT NULL,
    user_id VARCHAR(36) NOT NULL,
    read_time TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT uk_sys_notice_read_user_notice UNIQUE (user_id, notice_id),
    CONSTRAINT fk_sys_notice_read_notice FOREIGN KEY (notice_id) REFERENCES sys_notice(notice_id) ON DELETE CASCADE,
    CONSTRAINT fk_sys_notice_read_user FOREIGN KEY (user_id) REFERENCES sys_user(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_sys_notice_read_notice_time ON sys_notice_read (notice_id, read_time DESC);

INSERT INTO sys_menu (
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark
)
VALUES
    ('110', '通知公告', '1', 11, '/dashboard/admin/notices', 'system/notice/index', '', 'Notice', FALSE, FALSE, 'C', '0', '0', 'system:notice:list', 'icon.notice', 'admin', CURRENT_TIMESTAMP, '通知公告菜单'),
    ('1100', '公告查询', '110', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:notice:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1101', '公告新增', '110', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:notice:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1102', '公告修改', '110', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:notice:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1103', '公告删除', '110', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:notice:remove', '#', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT '2', menu_id
FROM sys_menu
WHERE menu_id IN ('110', '1100', '1101', '1102', '1103');

INSERT INTO sys_dict_type (dict_id, dict_name, dict_type, status, create_by, create_time, remark)
VALUES
    ('notice-type', '通知类型', 'sys_notice_type', '0', 'admin', CURRENT_TIMESTAMP, '通知公告类型'),
    ('notice-status', '通知状态', 'sys_notice_status', '0', 'admin', CURRENT_TIMESTAMP, '通知公告状态');

INSERT INTO sys_dict_data (dict_code, dict_sort, dict_label, dict_value, dict_type, css_class, list_class, is_default, status, create_by, create_time, remark)
VALUES
    ('notice-type-notice', 1, '通知', '1', 'sys_notice_type', '', 'warning', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '通知'),
    ('notice-type-announcement', 2, '公告', '2', 'sys_notice_type', '', 'success', 'N', '0', 'admin', CURRENT_TIMESTAMP, '公告'),
    ('notice-status-normal', 1, '正常', '0', 'sys_notice_status', '', 'primary', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '正常状态'),
    ('notice-status-closed', 2, '关闭', '1', 'sys_notice_status', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '关闭状态');
