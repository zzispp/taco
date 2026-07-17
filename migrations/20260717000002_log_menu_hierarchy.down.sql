UPDATE sys_menu
SET parent_id = '3',
    order_num = 3
WHERE menu_id = '109';

UPDATE sys_menu
SET order_num = 3
WHERE menu_id = '114'
  AND parent_id = '111';
