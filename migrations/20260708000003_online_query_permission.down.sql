DELETE FROM sys_role_menu WHERE menu_id = '1071';
DELETE FROM sys_menu WHERE menu_id = '1071';
UPDATE sys_menu SET order_num = 1 WHERE menu_id = '1070';
