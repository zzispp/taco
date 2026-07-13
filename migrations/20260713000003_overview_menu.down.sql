UPDATE sys_menu
SET parent_id = '0', order_num = 0
WHERE menu_id = '2';

DELETE FROM sys_role_menu WHERE menu_id = '4';
DELETE FROM sys_menu WHERE menu_id = '4';
