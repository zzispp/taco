DROP INDEX IF EXISTS idx_sys_notice_read_cursor;
CREATE INDEX idx_sys_notice_read_notice_time
    ON sys_notice_read (notice_id, read_time DESC);

DROP INDEX IF EXISTS idx_sys_config_cursor;
DROP INDEX IF EXISTS idx_sys_dict_data_sort_cursor;
DROP INDEX IF EXISTS idx_sys_dict_data_cursor;
DROP INDEX IF EXISTS idx_sys_dict_type_cursor;
DROP INDEX IF EXISTS idx_sys_post_sort_cursor;
DROP INDEX IF EXISTS idx_sys_post_cursor;
DROP INDEX IF EXISTS idx_sys_dept_active_sort_cursor;
DROP INDEX IF EXISTS idx_sys_dept_active_cursor;
DROP INDEX IF EXISTS idx_sys_menu_sort_cursor;
DROP INDEX IF EXISTS idx_sys_menu_cursor;
DROP INDEX IF EXISTS idx_sys_role_active_sort_cursor;
DROP INDEX IF EXISTS idx_sys_role_active_cursor;
DROP INDEX IF EXISTS idx_sys_user_active_dept_cursor;
DROP INDEX IF EXISTS idx_sys_user_active_cursor;
