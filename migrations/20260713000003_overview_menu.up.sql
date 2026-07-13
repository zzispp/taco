INSERT INTO sys_menu (
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark
)
VALUES (
    '4', '概览', '0', 0, '/dashboard/overview', NULL, '', '',
    FALSE, FALSE, 'M', '0', '0', NULL, 'icon.dashboard', 'admin', CURRENT_TIMESTAMP, '概览目录'
);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT DISTINCT role_id, '4'
FROM sys_role_menu
WHERE menu_id = '2';

UPDATE sys_menu
SET parent_id = '4', order_num = 1
WHERE menu_id = '2';
