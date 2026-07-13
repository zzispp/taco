UPDATE sys_menu
SET parent_id = '1',
    order_num = CASE menu_id
        WHEN '107' THEN 8
        WHEN '108' THEN 9
        WHEN '109' THEN 10
    END
WHERE menu_id IN ('107', '108', '109');

DELETE FROM sys_role_menu WHERE menu_id = '3';
DELETE FROM sys_menu WHERE menu_id = '3';
