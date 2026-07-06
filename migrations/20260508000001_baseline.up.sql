CREATE TABLE sys_dept (
    dept_id VARCHAR(36) PRIMARY KEY,
    parent_id VARCHAR(36) NOT NULL DEFAULT '0',
    ancestors VARCHAR(200) NOT NULL DEFAULT '',
    dept_name VARCHAR(30) NOT NULL DEFAULT '',
    order_num BIGINT NOT NULL DEFAULT 0,
    leader VARCHAR(20) NULL,
    phone VARCHAR(20) NULL,
    email VARCHAR(50) NULL,
    status CHAR(1) NOT NULL DEFAULT '0',
    del_flag CHAR(1) NOT NULL DEFAULT '0',
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL
);

CREATE TABLE sys_role (
    role_id VARCHAR(36) PRIMARY KEY,
    role_name VARCHAR(30) NOT NULL,
    role_key VARCHAR(100) NOT NULL,
    role_sort BIGINT NOT NULL,
    data_scope CHAR(1) NOT NULL DEFAULT '1',
    menu_check_strictly BOOLEAN NOT NULL DEFAULT TRUE,
    dept_check_strictly BOOLEAN NOT NULL DEFAULT TRUE,
    status CHAR(1) NOT NULL,
    del_flag CHAR(1) NOT NULL DEFAULT '0',
    system BOOLEAN NOT NULL DEFAULT FALSE,
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE TABLE sys_user (
    user_id VARCHAR(36) PRIMARY KEY,
    dept_id VARCHAR(36) NULL,
    user_name VARCHAR(30) NOT NULL,
    nick_name VARCHAR(30) NOT NULL,
    user_type VARCHAR(2) NOT NULL DEFAULT '00',
    email VARCHAR(50) NOT NULL DEFAULT '',
    phonenumber VARCHAR(20) NULL,
    sex CHAR(1) NOT NULL DEFAULT '2',
    avatar VARCHAR(255) NULL,
    password VARCHAR(255) NOT NULL DEFAULT '',
    status CHAR(1) NOT NULL DEFAULT '0',
    del_flag CHAR(1) NOT NULL DEFAULT '0',
    login_ip VARCHAR(128) NOT NULL DEFAULT '',
    login_date TIMESTAMPTZ NULL,
    pwd_update_date TIMESTAMPTZ NULL,
    auth_source VARCHAR(50) NOT NULL DEFAULT 'local',
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE TABLE sys_post (
    post_id VARCHAR(36) PRIMARY KEY,
    post_code VARCHAR(64) NOT NULL,
    post_name VARCHAR(50) NOT NULL,
    post_sort BIGINT NOT NULL,
    status CHAR(1) NOT NULL,
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE TABLE sys_menu (
    menu_id VARCHAR(36) PRIMARY KEY,
    menu_name VARCHAR(50) NOT NULL,
    parent_id VARCHAR(36) NOT NULL DEFAULT '0',
    order_num BIGINT NOT NULL DEFAULT 0,
    path VARCHAR(200) NOT NULL DEFAULT '',
    component VARCHAR(255) NULL,
    query VARCHAR(255) NULL,
    route_name VARCHAR(50) NOT NULL DEFAULT '',
    is_frame BOOLEAN NOT NULL DEFAULT FALSE,
    is_cache BOOLEAN NOT NULL DEFAULT FALSE,
    menu_type CHAR(1) NOT NULL DEFAULT '',
    visible CHAR(1) NOT NULL DEFAULT '0',
    status CHAR(1) NOT NULL DEFAULT '0',
    perms VARCHAR(100) NULL,
    icon VARCHAR(100) NOT NULL DEFAULT '#',
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE TABLE sys_user_role (
    user_id VARCHAR(36) NOT NULL,
    role_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (user_id, role_id)
);

CREATE TABLE sys_role_menu (
    role_id VARCHAR(36) NOT NULL,
    menu_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (role_id, menu_id)
);

CREATE TABLE sys_role_dept (
    role_id VARCHAR(36) NOT NULL,
    dept_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (role_id, dept_id)
);

CREATE TABLE sys_user_post (
    user_id VARCHAR(36) NOT NULL,
    post_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (user_id, post_id)
);

CREATE TABLE sys_dict_type (
    dict_id VARCHAR(36) PRIMARY KEY,
    dict_name VARCHAR(100) NOT NULL DEFAULT '',
    dict_type VARCHAR(100) NOT NULL DEFAULT '',
    status CHAR(1) NOT NULL DEFAULT '0',
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE TABLE sys_dict_data (
    dict_code VARCHAR(36) PRIMARY KEY,
    dict_sort BIGINT NOT NULL DEFAULT 0,
    dict_label VARCHAR(100) NOT NULL DEFAULT '',
    dict_value VARCHAR(100) NOT NULL DEFAULT '',
    dict_type VARCHAR(100) NOT NULL DEFAULT '',
    css_class VARCHAR(100) NULL,
    list_class VARCHAR(100) NULL,
    is_default CHAR(1) NOT NULL DEFAULT 'N',
    status CHAR(1) NOT NULL DEFAULT '0',
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE TABLE sys_config (
    config_id VARCHAR(36) PRIMARY KEY,
    config_name VARCHAR(100) NOT NULL DEFAULT '',
    config_key VARCHAR(100) NOT NULL DEFAULT '',
    config_value TEXT NOT NULL DEFAULT '',
    config_type CHAR(1) NOT NULL DEFAULT 'N',
    create_by VARCHAR(64) NOT NULL DEFAULT '',
    create_time TIMESTAMPTZ NOT NULL,
    update_by VARCHAR(64) NOT NULL DEFAULT '',
    update_time TIMESTAMPTZ NULL,
    remark VARCHAR(500) NULL
);

CREATE UNIQUE INDEX idx_sys_user_name ON sys_user (user_name);
CREATE UNIQUE INDEX idx_sys_user_phone ON sys_user (phonenumber) WHERE phonenumber IS NOT NULL;
CREATE INDEX idx_sys_user_dept ON sys_user (dept_id);
CREATE UNIQUE INDEX idx_sys_role_key ON sys_role (role_key);
CREATE INDEX idx_sys_menu_parent ON sys_menu (parent_id);
CREATE INDEX idx_sys_menu_perms ON sys_menu (perms);
CREATE UNIQUE INDEX idx_sys_post_code ON sys_post (post_code);
CREATE UNIQUE INDEX idx_sys_dict_type ON sys_dict_type (dict_type);
CREATE INDEX idx_sys_dict_data_type ON sys_dict_data (dict_type);
CREATE UNIQUE INDEX idx_sys_config_key ON sys_config (config_key);

ALTER TABLE sys_user_role ADD CONSTRAINT fk_sys_user_role_user FOREIGN KEY (user_id) REFERENCES sys_user(user_id) ON DELETE CASCADE;
ALTER TABLE sys_user_role ADD CONSTRAINT fk_sys_user_role_role FOREIGN KEY (role_id) REFERENCES sys_role(role_id) ON DELETE CASCADE;
ALTER TABLE sys_role_menu ADD CONSTRAINT fk_sys_role_menu_role FOREIGN KEY (role_id) REFERENCES sys_role(role_id) ON DELETE CASCADE;
ALTER TABLE sys_role_menu ADD CONSTRAINT fk_sys_role_menu_menu FOREIGN KEY (menu_id) REFERENCES sys_menu(menu_id) ON DELETE CASCADE;
ALTER TABLE sys_role_dept ADD CONSTRAINT fk_sys_role_dept_role FOREIGN KEY (role_id) REFERENCES sys_role(role_id) ON DELETE CASCADE;
ALTER TABLE sys_role_dept ADD CONSTRAINT fk_sys_role_dept_dept FOREIGN KEY (dept_id) REFERENCES sys_dept(dept_id) ON DELETE CASCADE;
ALTER TABLE sys_user_post ADD CONSTRAINT fk_sys_user_post_user FOREIGN KEY (user_id) REFERENCES sys_user(user_id) ON DELETE CASCADE;
ALTER TABLE sys_user_post ADD CONSTRAINT fk_sys_user_post_post FOREIGN KEY (post_id) REFERENCES sys_post(post_id) ON DELETE CASCADE;
