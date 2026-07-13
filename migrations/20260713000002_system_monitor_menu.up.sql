INSERT INTO sys_menu (
    menu_id, menu_name, parent_id, order_num, path, component, query, route_name,
    is_frame, is_cache, menu_type, visible, status, perms, icon, create_by, create_time, remark
)
VALUES (
    '3', '系统监控', '0', 2, '/dashboard/monitor', NULL, '', '',
    FALSE, FALSE, 'M', '0', '0', NULL, 'icon.monitor', 'admin', CURRENT_TIMESTAMP, '系统监控目录'
);

INSERT INTO sys_role_menu (role_id, menu_id)
SELECT DISTINCT role_id, '3'
FROM sys_role_menu
WHERE menu_id IN ('107', '108', '109');

UPDATE sys_menu
SET parent_id = '3',
    order_num = CASE menu_id
        WHEN '107' THEN 1
        WHEN '108' THEN 2
        WHEN '109' THEN 3
    END
WHERE menu_id IN ('107', '108', '109');
