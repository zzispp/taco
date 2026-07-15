UPDATE sys_user
SET email = LOWER(BTRIM(email))
WHERE email <> LOWER(BTRIM(email));

DROP INDEX IF EXISTS idx_sys_user_name;
DROP INDEX IF EXISTS idx_sys_user_phone;

CREATE UNIQUE INDEX idx_sys_user_name_active
    ON sys_user (user_name)
    WHERE del_flag = '0';

CREATE UNIQUE INDEX idx_sys_user_email_active_ci
    ON sys_user (LOWER(email))
    WHERE del_flag = '0' AND BTRIM(email) <> '';

CREATE UNIQUE INDEX idx_sys_user_phone_active
    ON sys_user (phonenumber)
    WHERE del_flag = '0' AND phonenumber IS NOT NULL;

CREATE INDEX idx_sys_user_status ON sys_user (status) WHERE del_flag = '0';
CREATE INDEX idx_sys_user_create_time ON sys_user (create_time) WHERE del_flag = '0';

CREATE UNIQUE INDEX idx_sys_role_name ON sys_role (role_name);

CREATE UNIQUE INDEX idx_sys_menu_parent_name ON sys_menu (parent_id, menu_name);
CREATE UNIQUE INDEX idx_sys_menu_parent_path ON sys_menu (parent_id, path) WHERE path NOT IN ('', '#');
CREATE UNIQUE INDEX idx_sys_menu_route_name ON sys_menu (route_name) WHERE route_name <> '';

ALTER TABLE sys_role
    ADD CONSTRAINT chk_sys_role_data_scope CHECK (data_scope IN ('1', '2', '3', '4', '5'));
