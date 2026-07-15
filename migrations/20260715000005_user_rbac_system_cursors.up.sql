CREATE INDEX idx_sys_user_active_cursor
    ON sys_user (create_time ASC, user_id ASC)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_user_active_dept_cursor
    ON sys_user (dept_id, create_time ASC, user_id ASC)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_role_active_cursor
    ON sys_role (create_time ASC, role_id ASC)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_role_active_sort_cursor
    ON sys_role (role_sort ASC, role_id ASC)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_menu_cursor
    ON sys_menu (create_time ASC, menu_id ASC);

CREATE INDEX idx_sys_menu_sort_cursor
    ON sys_menu (parent_id ASC, order_num ASC, menu_id ASC);

CREATE INDEX idx_sys_dept_active_cursor
    ON sys_dept (create_time ASC, dept_id ASC)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_dept_active_sort_cursor
    ON sys_dept (parent_id ASC, order_num ASC, dept_id ASC)
    WHERE del_flag = '0';

CREATE INDEX idx_sys_post_cursor
    ON sys_post (create_time ASC, post_id ASC);

CREATE INDEX idx_sys_post_sort_cursor
    ON sys_post (post_sort ASC, post_id ASC);

CREATE INDEX idx_sys_dict_type_cursor
    ON sys_dict_type (create_time ASC, dict_id ASC);

CREATE INDEX idx_sys_dict_data_cursor
    ON sys_dict_data (create_time ASC, dict_code ASC);

CREATE INDEX idx_sys_dict_data_sort_cursor
    ON sys_dict_data (dict_sort ASC, dict_code ASC);

CREATE INDEX idx_sys_config_cursor
    ON sys_config (create_time ASC, config_id ASC);

DROP INDEX idx_sys_notice_read_notice_time;

CREATE INDEX idx_sys_notice_read_cursor
    ON sys_notice_read (notice_id, read_time DESC, user_id DESC);
