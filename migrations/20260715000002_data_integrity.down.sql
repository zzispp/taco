ALTER TABLE sys_role DROP CONSTRAINT IF EXISTS chk_sys_role_data_scope;

DROP INDEX IF EXISTS idx_sys_menu_route_name;
DROP INDEX IF EXISTS idx_sys_menu_parent_path;
DROP INDEX IF EXISTS idx_sys_menu_parent_name;
DROP INDEX IF EXISTS idx_sys_role_name;
DROP INDEX IF EXISTS idx_sys_user_create_time;
DROP INDEX IF EXISTS idx_sys_user_status;
DROP INDEX IF EXISTS idx_sys_user_phone_active;
DROP INDEX IF EXISTS idx_sys_user_email_active_ci;
DROP INDEX IF EXISTS idx_sys_user_name_active;

CREATE UNIQUE INDEX idx_sys_user_name ON sys_user (user_name);
CREATE UNIQUE INDEX idx_sys_user_phone ON sys_user (phonenumber) WHERE phonenumber IS NOT NULL;
