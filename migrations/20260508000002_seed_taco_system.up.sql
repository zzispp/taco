INSERT INTO sys_dept (dept_id, parent_id, ancestors, dept_name, order_num, leader, phone, email, status, del_flag, create_by, create_time)
VALUES
    ('100', '0', '0', 'Taco科技', 0, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('101', '100', '0,100', '深圳总公司', 1, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('102', '100', '0,100', '长沙分公司', 2, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('103', '101', '0,100,101', '研发部门', 1, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('104', '101', '0,100,101', '市场部门', 2, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('105', '101', '0,100,101', '测试部门', 3, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('106', '101', '0,100,101', '财务部门', 4, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('107', '101', '0,100,101', '运维部门', 5, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('108', '102', '0,100,102', '市场部门', 1, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP),
    ('109', '102', '0,100,102', '财务部门', 2, 'taco', '15888888888', 'taco@example.com', '0', '0', 'admin', CURRENT_TIMESTAMP);

INSERT INTO sys_role (role_id, role_name, role_key, role_sort, data_scope, menu_check_strictly, dept_check_strictly, status, del_flag, system, create_by, create_time, remark)
VALUES ('admin', '系统管理员', 'admin', 1, '1', TRUE, TRUE, '0', '0', TRUE, 'admin', CURRENT_TIMESTAMP, '系统管理员');

INSERT INTO sys_post (post_id, post_code, post_name, post_sort, status, create_by, create_time, remark)
VALUES
    ('1', 'ceo', '董事长', 1, '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('2', 'se', '项目经理', 2, '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('3', 'hr', '人力资源', 3, '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('4', 'user', '普通员工', 4, '0', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark)
VALUES
    ('2', '仪表盘', '0', 0, '/dashboard', 'dashboard/index', '', 'Dashboard', FALSE, FALSE, 'C', '0', '0', 'system:dashboard:view', 'icon.dashboard', 'admin', CURRENT_TIMESTAMP, '仪表盘菜单'),
    ('1', '系统管理', '0', 1, '/dashboard/admin', NULL, '', '', FALSE, FALSE, 'M', '0', '0', NULL, 'icon.folder', 'admin', CURRENT_TIMESTAMP, '系统管理目录'),
    ('100', '用户管理', '1', 1, '/dashboard/admin/users', 'system/user/index', '', 'User', FALSE, FALSE, 'C', '0', '0', 'system:user:list', 'icon.user', 'admin', CURRENT_TIMESTAMP, '用户管理菜单'),
    ('101', '角色管理', '1', 2, '/dashboard/admin/roles', 'system/role/index', '', 'Role', FALSE, FALSE, 'C', '0', '0', 'system:role:list', 'icon.lock', 'admin', CURRENT_TIMESTAMP, '角色管理菜单'),
    ('102', '菜单管理', '1', 3, '/dashboard/admin/menus', 'system/menu/index', '', 'Menu', FALSE, FALSE, 'C', '0', '0', 'system:menu:list', 'icon.menu', 'admin', CURRENT_TIMESTAMP, '菜单管理菜单'),
    ('103', '部门管理', '1', 4, '/dashboard/admin/depts', 'system/dept/index', '', 'Dept', FALSE, FALSE, 'C', '0', '0', 'system:dept:list', 'icon.folder', 'admin', CURRENT_TIMESTAMP, '部门管理菜单'),
    ('104', '岗位管理', '1', 5, '/dashboard/admin/posts', 'system/post/index', '', 'Post', FALSE, FALSE, 'C', '0', '0', 'system:post:list', 'icon.file', 'admin', CURRENT_TIMESTAMP, '岗位管理菜单'),
    ('105', '字典管理', '1', 6, '/dashboard/admin/dicts', 'system/dict/index', '', 'Dict', FALSE, FALSE, 'C', '0', '0', 'system:dict:list', 'icon.analytics', 'admin', CURRENT_TIMESTAMP, '字典管理菜单'),
    ('106', '参数设置', '1', 7, '/dashboard/admin/configs', 'system/config/index', '', 'Config', FALSE, FALSE, 'C', '0', '0', 'system:config:list', 'icon.kanban', 'admin', CURRENT_TIMESTAMP, '参数设置菜单'),
    ('1000', '用户查询', '100', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1001', '用户新增', '100', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1002', '用户修改', '100', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1003', '用户删除', '100', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1004', '用户导出', '100', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1005', '用户导入', '100', 6, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:import', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1006', '用户重置密码', '100', 7, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:user:resetPwd', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1010', '角色查询', '101', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:role:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1011', '角色新增', '101', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:role:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1012', '角色修改', '101', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:role:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1013', '角色删除', '101', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:role:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1014', '角色导出', '101', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:role:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1020', '菜单查询', '102', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:menu:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1021', '菜单新增', '102', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:menu:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1022', '菜单修改', '102', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:menu:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1023', '菜单删除', '102', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:menu:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1030', '部门查询', '103', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dept:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1031', '部门新增', '103', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dept:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1032', '部门修改', '103', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dept:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1033', '部门删除', '103', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dept:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1040', '岗位查询', '104', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:post:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1041', '岗位新增', '104', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:post:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1042', '岗位修改', '104', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:post:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1043', '岗位删除', '104', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:post:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1044', '岗位导出', '104', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:post:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1050', '字典查询', '105', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dict:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1051', '字典新增', '105', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dict:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1052', '字典修改', '105', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dict:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1053', '字典删除', '105', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dict:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1054', '字典导出', '105', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:dict:export', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1060', '参数查询', '106', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:config:query', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1061', '参数新增', '106', 2, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:config:add', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1062', '参数修改', '106', 3, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:config:edit', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1063', '参数删除', '106', 4, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:config:remove', '#', 'admin', CURRENT_TIMESTAMP, ''),
    ('1064', '参数导出', '106', 5, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:config:export', '#', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_role_menu (role_id, menu_id) SELECT 'admin', menu_id FROM sys_menu;

INSERT INTO sys_dict_type (dict_id, dict_name, dict_type, status, create_by, create_time, remark)
VALUES
    ('1', '用户性别', 'sys_user_sex', '0', 'admin', CURRENT_TIMESTAMP, '用户性别列表'),
    ('2', '菜单状态', 'sys_show_hide', '0', 'admin', CURRENT_TIMESTAMP, '菜单状态列表'),
    ('3', '系统开关', 'sys_normal_disable', '0', 'admin', CURRENT_TIMESTAMP, '系统开关列表'),
    ('6', '系统是否', 'sys_yes_no', '0', 'admin', CURRENT_TIMESTAMP, '系统是否列表'),
    ('10', '系统状态', 'sys_common_status', '0', 'admin', CURRENT_TIMESTAMP, '登录状态列表');

INSERT INTO sys_dict_data (dict_code, dict_sort, dict_label, dict_value, dict_type, css_class, list_class, is_default, status, create_by, create_time, remark)
VALUES
    ('1', 1, '男', '0', 'sys_user_sex', '', '', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '性别男'),
    ('2', 2, '女', '1', 'sys_user_sex', '', '', 'N', '0', 'admin', CURRENT_TIMESTAMP, '性别女'),
    ('3', 3, '未知', '2', 'sys_user_sex', '', '', 'N', '0', 'admin', CURRENT_TIMESTAMP, '性别未知'),
    ('4', 1, '显示', '0', 'sys_show_hide', '', 'primary', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '显示菜单'),
    ('5', 2, '隐藏', '1', 'sys_show_hide', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '隐藏菜单'),
    ('6', 1, '正常', '0', 'sys_normal_disable', '', 'primary', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '正常状态'),
    ('7', 2, '停用', '1', 'sys_normal_disable', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '停用状态'),
    ('12', 1, '是', 'Y', 'sys_yes_no', '', 'primary', 'Y', '0', 'admin', CURRENT_TIMESTAMP, '系统默认是'),
    ('13', 2, '否', 'N', 'sys_yes_no', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '系统默认否'),
    ('28', 1, '成功', '0', 'sys_common_status', '', 'primary', 'N', '0', 'admin', CURRENT_TIMESTAMP, '正常状态'),
    ('29', 2, '失败', '1', 'sys_common_status', '', 'danger', 'N', '0', 'admin', CURRENT_TIMESTAMP, '停用状态');

INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, create_by, create_time, remark)
VALUES
    ('1', '主框架页-默认皮肤样式名称', 'sys.index.skinName', 'skin-blue', 'Y', 'admin', CURRENT_TIMESTAMP, '默认皮肤样式。可选值：skin-blue 蓝色、skin-green 绿色、skin-purple 紫色、skin-red 红色、skin-yellow 黄色。'),
    ('3', '主框架页-默认主题', 'sys.index.modeTheme', 'theme-light', 'Y', 'admin', CURRENT_TIMESTAMP, '默认主题模式。可选值：theme-dark 深色主题、theme-light 浅色主题。'),
    ('4', '账号自助-验证码配置', 'sys.account.captchaConfig', '{"enabled":true,"provider":"cap","providers":{"cap":{"challenge_count":50,"challenge_size":32,"challenge_difficulty":4,"challenge_ttl_seconds":600,"redeemed_token_ttl_seconds":1200},"cloudflare_turnstile":{"site_key":"","theme":"auto","size":"normal"}}}', 'Y', 'admin', CURRENT_TIMESTAMP, '验证码非敏感运行时配置 JSON。enabled 控制开关，provider 选择 cap/cloudflare_turnstile，providers.cap 配置 PoW 难度和 TTL，providers.cloudflare_turnstile 配置 site_key、theme、size；Turnstile 私钥由部署 YAML 单独提供。'),
    ('5', '账号自助-是否开启用户注册功能', 'sys.account.registerUser', 'false', 'Y', 'admin', CURRENT_TIMESTAMP, '是否允许账号自助注册。true 允许注册，false 禁止注册。');
