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
VALUES
    ('1', '超级管理员', 'admin', 1, '1', TRUE, TRUE, '0', '0', TRUE, 'admin', CURRENT_TIMESTAMP, '超级管理员'),
    ('2', '普通角色', 'common', 2, '2', TRUE, TRUE, '0', '0', FALSE, 'admin', CURRENT_TIMESTAMP, '普通角色');

INSERT INTO sys_post (post_id, post_code, post_name, post_sort, status, create_by, create_time, remark)
VALUES
    ('1', 'ceo', '董事长', 1, '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('2', 'se', '项目经理', 2, '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('3', 'hr', '人力资源', 3, '0', 'admin', CURRENT_TIMESTAMP, ''),
    ('4', 'user', '普通员工', 4, '0', 'admin', CURRENT_TIMESTAMP, '');

INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark)
VALUES
    ('1', '系统管理', '0', 1, 'system', NULL, '', '', FALSE, FALSE, 'M', '0', '0', NULL, 'icon.dashboard', 'admin', CURRENT_TIMESTAMP, '系统管理目录'),
    ('100', '用户管理', '1', 1, 'user', 'system/user/index', '', 'User', FALSE, FALSE, 'C', '0', '0', 'system:user:list', 'icon.user', 'admin', CURRENT_TIMESTAMP, '用户管理菜单'),
    ('101', '角色管理', '1', 2, 'role', 'system/role/index', '', 'Role', FALSE, FALSE, 'C', '0', '0', 'system:role:list', 'icon.lock', 'admin', CURRENT_TIMESTAMP, '角色管理菜单'),
    ('102', '菜单管理', '1', 3, 'menu', 'system/menu/index', '', 'Menu', FALSE, FALSE, 'C', '0', '0', 'system:menu:list', 'icon.menu', 'admin', CURRENT_TIMESTAMP, '菜单管理菜单'),
    ('103', '部门管理', '1', 4, 'dept', 'system/dept/index', '', 'Dept', FALSE, FALSE, 'C', '0', '0', 'system:dept:list', 'icon.folder', 'admin', CURRENT_TIMESTAMP, '部门管理菜单'),
    ('104', '岗位管理', '1', 5, 'post', 'system/post/index', '', 'Post', FALSE, FALSE, 'C', '0', '0', 'system:post:list', 'icon.file', 'admin', CURRENT_TIMESTAMP, '岗位管理菜单'),
    ('105', '字典管理', '1', 6, 'dict', 'system/dict/index', '', 'Dict', FALSE, FALSE, 'C', '0', '0', 'system:dict:list', 'icon.analytics', 'admin', CURRENT_TIMESTAMP, '字典管理菜单'),
    ('106', '参数设置', '1', 7, 'config', 'system/config/index', '', 'Config', FALSE, FALSE, 'C', '0', '0', 'system:config:list', 'icon.kanban', 'admin', CURRENT_TIMESTAMP, '参数设置菜单'),
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

INSERT INTO sys_user (user_id, dept_id, user_name, nick_name, user_type, email, phonenumber, sex, avatar, password, status, del_flag, login_ip, login_date, pwd_update_date, auth_source, email_verified, create_by, create_time, remark)
VALUES
    ('1', '103', 'admin', 'taco', '00', 'admin@taco.local', '15888888888', '1', '', '$argon2id$v=19$m=19456,t=2,p=1$FpN5fcXCVNOVBGybU6xVBA$1y5zUgTGlohI/RVCXKyckyv1CCyuzVhVMagjJux9rUA', '0', '0', '127.0.0.1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 'local', TRUE, 'admin', CURRENT_TIMESTAMP, '管理员'),
    ('2', '105', 'taco', 'taco', '00', 'taco@example.com', '15666666666', '1', '', '$argon2id$v=19$m=19456,t=2,p=1$FpN5fcXCVNOVBGybU6xVBA$1y5zUgTGlohI/RVCXKyckyv1CCyuzVhVMagjJux9rUA', '0', '0', '127.0.0.1', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 'local', TRUE, 'admin', CURRENT_TIMESTAMP, '测试员');

INSERT INTO sys_user_role (user_id, role_id) VALUES ('1', '1'), ('2', '2');
INSERT INTO sys_user_post (user_id, post_id) VALUES ('1', '1'), ('2', '2');
INSERT INTO sys_role_dept (role_id, dept_id) VALUES ('2', '100'), ('2', '101'), ('2', '105');
INSERT INTO sys_role_menu (role_id, menu_id) SELECT '2', menu_id FROM sys_menu;

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
    ('1', '主框架页-默认皮肤样式名称', 'sys.index.skinName', 'skin-blue', 'Y', 'admin', CURRENT_TIMESTAMP, '蓝色 skin-blue、绿色 skin-green、紫色 skin-purple、红色 skin-red、黄色 skin-yellow'),
    ('2', '用户管理-账号初始密码', 'sys.user.initPassword', '123456', 'Y', 'admin', CURRENT_TIMESTAMP, '初始化密码 123456'),
    ('3', '主框架页-侧边栏主题', 'sys.index.sideTheme', 'theme-dark', 'Y', 'admin', CURRENT_TIMESTAMP, '深色主题theme-dark，浅色主题theme-light'),
    ('4', '账号自助-验证码开关', 'sys.account.captchaEnabled', 'true', 'Y', 'admin', CURRENT_TIMESTAMP, '是否开启验证码功能'),
    ('5', '账号自助-是否开启用户注册功能', 'sys.account.registerUser', 'false', 'Y', 'admin', CURRENT_TIMESTAMP, '是否开启注册用户功能');
