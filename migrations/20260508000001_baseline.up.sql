CREATE TABLE users (
    id VARCHAR(36) PRIMARY KEY,
    username VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(100) NOT NULL,
    is_active BOOLEAN NOT NULL,
    is_deleted BOOLEAN NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_login_at TIMESTAMPTZ NULL,
    auth_source VARCHAR(50) NOT NULL,
    email_verified BOOLEAN NOT NULL
);

CREATE TABLE roles (
    code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    enabled BOOLEAN NOT NULL,
    system BOOLEAN NOT NULL,
    sort_order BIGINT NOT NULL
);

CREATE TABLE api_permissions (
    id VARCHAR(36) PRIMARY KEY,
    code TEXT NOT NULL,
    method TEXT NOT NULL,
    path_pattern TEXT NOT NULL,
    name TEXT NOT NULL,
    "group" TEXT NOT NULL,
    enabled BOOLEAN NOT NULL,
    system BOOLEAN NOT NULL
);

CREATE TABLE menu_sections (
    id VARCHAR(36) PRIMARY KEY,
    code TEXT NOT NULL,
    subheader TEXT NOT NULL,
    sort_order BIGINT NOT NULL,
    enabled BOOLEAN NOT NULL
);

CREATE TABLE menu_items (
    id VARCHAR(36) PRIMARY KEY,
    section_id VARCHAR(36) NOT NULL,
    parent_id VARCHAR(36) NULL,
    code TEXT NOT NULL,
    title TEXT NOT NULL,
    route_path TEXT NOT NULL,
    icon TEXT NULL,
    caption TEXT NULL,
    deep_match BOOLEAN NOT NULL,
    sort_order BIGINT NOT NULL,
    enabled BOOLEAN NOT NULL
);

CREATE TABLE role_api_permissions (
    role_code TEXT NOT NULL,
    api_permission_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (role_code, api_permission_id)
);

CREATE TABLE role_menu_permissions (
    role_code TEXT NOT NULL,
    menu_item_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (role_code, menu_item_id)
);

CREATE UNIQUE INDEX index_users_by_username ON users (username);
CREATE UNIQUE INDEX index_users_by_email ON users (email);
CREATE UNIQUE INDEX index_api_permissions_by_code ON api_permissions (code);
CREATE UNIQUE INDEX index_menu_sections_by_code ON menu_sections (code);
CREATE INDEX index_menu_items_by_section_id ON menu_items (section_id);
CREATE UNIQUE INDEX index_menu_items_by_code ON menu_items (code);

INSERT INTO roles (code, name, description, enabled, system, sort_order)
VALUES
    ('admin', 'Administrator', 'Built-in administrator role', TRUE, TRUE, 0),
    ('user', 'User', 'Default signed-up user role', TRUE, TRUE, 10);

INSERT INTO api_permissions (id, code, method, path_pattern, name, "group", enabled, system)
VALUES
    ('00000000-0000-7000-8000-000000000301', 'auth_me', 'GET', '/api/auth/me', 'Current user', 'Auth', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000302', 'navbar_read', 'GET', '/api/navbar', 'Navbar', 'System', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000303', 'users_read', 'GET', '/api/users', 'List users', 'Users', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000304', 'users_create', 'POST', '/api/users', 'Create user', 'Users', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000305', 'users_update', 'PUT', '/api/users/{id}', 'Update user', 'Users', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000306', 'users_delete', 'DELETE', '/api/users/{id}', 'Delete user', 'Users', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000307', 'roles_read', 'GET', '/api/rbac/roles', 'List roles', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000308', 'roles_create', 'POST', '/api/rbac/roles', 'Create role', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000309', 'roles_update', 'PUT', '/api/rbac/roles/{code}', 'Update role', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000310', 'roles_delete', 'DELETE', '/api/rbac/roles/{code}', 'Delete role', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000311', 'role_apis_read', 'GET', '/api/rbac/roles/{code}/apis', 'Read role API bindings', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000312', 'role_apis_update', 'PUT', '/api/rbac/roles/{code}/apis', 'Update role API bindings', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000313', 'role_menus_read', 'GET', '/api/rbac/roles/{code}/menus', 'Read role menu bindings', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000314', 'role_menus_update', 'PUT', '/api/rbac/roles/{code}/menus', 'Update role menu bindings', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000315', 'apis_read', 'GET', '/api/rbac/apis', 'List API permissions', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000316', 'apis_create', 'POST', '/api/rbac/apis', 'Create API permission', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000317', 'apis_update', 'PUT', '/api/rbac/apis/{id}', 'Update API permission', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000318', 'apis_delete', 'DELETE', '/api/rbac/apis/{id}', 'Delete API permission', 'RBAC', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000319', 'menu_sections_read', 'GET', '/api/rbac/menu-sections', 'List menu sections', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000320', 'menu_sections_create', 'POST', '/api/rbac/menu-sections', 'Create menu section', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000321', 'menu_sections_update', 'PUT', '/api/rbac/menu-sections/{id}', 'Update menu section', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000322', 'menu_sections_delete', 'DELETE', '/api/rbac/menu-sections/{id}', 'Delete menu section', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000323', 'menu_items_read', 'GET', '/api/rbac/menu-items', 'List menu items', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000324', 'menu_items_create', 'POST', '/api/rbac/menu-items', 'Create menu item', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000325', 'menu_items_update', 'PUT', '/api/rbac/menu-items/{id}', 'Update menu item', 'Menus', TRUE, TRUE),
    ('00000000-0000-7000-8000-000000000326', 'menu_items_delete', 'DELETE', '/api/rbac/menu-items/{id}', 'Delete menu item', 'Menus', TRUE, TRUE);

INSERT INTO menu_sections (id, code, subheader, sort_order, enabled)
VALUES
    ('00000000-0000-7000-8000-000000000103', 'system_management', 'System Management', 0, TRUE);

INSERT INTO menu_items (
    id, section_id, parent_id, code, title, route_path, icon, caption, deep_match, sort_order, enabled
)
VALUES
    ('00000000-0000-7000-8000-000000000203', '00000000-0000-7000-8000-000000000103', NULL, 'admin_users', 'User Management', '/dashboard/admin/users', 'icon.user', NULL, TRUE, 0, TRUE),
    ('00000000-0000-7000-8000-000000000204', '00000000-0000-7000-8000-000000000103', NULL, 'admin_roles', 'Role Management', '/dashboard/admin/roles', 'icon.lock', NULL, TRUE, 10, TRUE),
    ('00000000-0000-7000-8000-000000000205', '00000000-0000-7000-8000-000000000103', NULL, 'admin_apis', 'API Management', '/dashboard/admin/apis', 'icon.menu', NULL, TRUE, 20, TRUE),
    ('00000000-0000-7000-8000-000000000206', '00000000-0000-7000-8000-000000000103', NULL, 'admin_menus', 'Menu Management', '/dashboard/admin/menus', 'icon.menu', NULL, TRUE, 30, TRUE);

INSERT INTO role_api_permissions (role_code, api_permission_id)
VALUES
    ('admin', '00000000-0000-7000-8000-000000000301'),
    ('admin', '00000000-0000-7000-8000-000000000302'),
    ('admin', '00000000-0000-7000-8000-000000000303'),
    ('admin', '00000000-0000-7000-8000-000000000304'),
    ('admin', '00000000-0000-7000-8000-000000000305'),
    ('admin', '00000000-0000-7000-8000-000000000306'),
    ('admin', '00000000-0000-7000-8000-000000000307'),
    ('admin', '00000000-0000-7000-8000-000000000308'),
    ('admin', '00000000-0000-7000-8000-000000000309'),
    ('admin', '00000000-0000-7000-8000-000000000310'),
    ('admin', '00000000-0000-7000-8000-000000000311'),
    ('admin', '00000000-0000-7000-8000-000000000312'),
    ('admin', '00000000-0000-7000-8000-000000000313'),
    ('admin', '00000000-0000-7000-8000-000000000314'),
    ('admin', '00000000-0000-7000-8000-000000000315'),
    ('admin', '00000000-0000-7000-8000-000000000316'),
    ('admin', '00000000-0000-7000-8000-000000000317'),
    ('admin', '00000000-0000-7000-8000-000000000318'),
    ('admin', '00000000-0000-7000-8000-000000000319'),
    ('admin', '00000000-0000-7000-8000-000000000320'),
    ('admin', '00000000-0000-7000-8000-000000000321'),
    ('admin', '00000000-0000-7000-8000-000000000322'),
    ('admin', '00000000-0000-7000-8000-000000000323'),
    ('admin', '00000000-0000-7000-8000-000000000324'),
    ('admin', '00000000-0000-7000-8000-000000000325'),
    ('admin', '00000000-0000-7000-8000-000000000326'),
    ('user', '00000000-0000-7000-8000-000000000301'),
    ('user', '00000000-0000-7000-8000-000000000302');

INSERT INTO role_menu_permissions (role_code, menu_item_id)
VALUES
    ('admin', '00000000-0000-7000-8000-000000000203'),
    ('admin', '00000000-0000-7000-8000-000000000204'),
    ('admin', '00000000-0000-7000-8000-000000000205'),
    ('admin', '00000000-0000-7000-8000-000000000206');
