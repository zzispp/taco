UPDATE sys_menu
SET icon = CASE menu_id
    WHEN '103' THEN 'icon.dept'
    WHEN '104' THEN 'icon.post'
    WHEN '105' THEN 'icon.dict'
    WHEN '106' THEN 'icon.config'
    WHEN '107' THEN 'icon.online'
    ELSE icon
END
WHERE menu_id IN ('103', '104', '105', '106', '107');
