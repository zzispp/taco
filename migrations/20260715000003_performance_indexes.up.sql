CREATE INDEX idx_sys_user_role_role_user ON sys_user_role (role_id, user_id);
CREATE INDEX idx_sys_user_post_post_user ON sys_user_post (post_id, user_id);
CREATE INDEX idx_sys_role_menu_menu_role ON sys_role_menu (menu_id, role_id);
CREATE INDEX idx_sys_role_dept_dept_role ON sys_role_dept (dept_id, role_id);

CREATE INDEX idx_sys_user_active_status_create_time
    ON sys_user (status, create_time, user_id)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_user_session_login_time
    ON sys_user_session (login_time DESC, token_id ASC);
