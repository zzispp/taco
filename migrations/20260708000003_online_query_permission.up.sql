INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, component, query, route_name, is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark)
VALUES ('1071', '在线用户查询', '107', 1, '#', '', '', '', FALSE, FALSE, 'F', '0', '0', 'system:online:query', '#', 'admin', CURRENT_TIMESTAMP, '')
ON CONFLICT (menu_id) DO NOTHING;

UPDATE sys_menu SET order_num = 2 WHERE menu_id = '1070';

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT '2', '1071'
WHERE EXISTS (SELECT 1 FROM sys_menu WHERE menu_id = '1071')
ON CONFLICT (role_id, menu_id) DO NOTHING;
