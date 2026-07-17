DO $log_menu_hierarchy$
BEGIN
    UPDATE sys_menu
    SET parent_id = '111',
        order_num = 3
    WHERE menu_id = '109';

    IF NOT FOUND THEN
        RAISE EXCEPTION 'scheduler log menu 109 is required before moving it under log management';
    END IF;

    UPDATE sys_menu
    SET order_num = 4
    WHERE menu_id = '114'
      AND parent_id = '111';

    IF NOT FOUND THEN
        RAISE EXCEPTION 'system log menu 114 is required before resequencing log management';
    END IF;
END;
$log_menu_hierarchy$;
