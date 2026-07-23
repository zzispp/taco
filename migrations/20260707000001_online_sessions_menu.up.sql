INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark)
VALUES
    ('107', '在线用户', '1', 8, '/dashboard/admin/online', 'system/online/index', '', 'Online', FALSE, FALSE, 'C', '0', '0', 'system:online:list', 'icon.user', 'admin', CURRENT_TIMESTAMP, '在线用户菜单'),
    ('1070', '在线用户强退', '107', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:online:forceLogout', '#', 'admin', CURRENT_TIMESTAMP, '')
ON CONFLICT (menu_id) DO NOTHING;

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT 'admin', menu_id
FROM sys_menu
WHERE menu_id IN ('107', '1070')
ON CONFLICT (role_id, menu_id) DO NOTHING;
