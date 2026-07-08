UPDATE sys_menu
SET icon = CASE menu_id
    WHEN '103' THEN 'icon.folder'
    WHEN '104' THEN 'icon.file'
    WHEN '105' THEN 'icon.analytics'
    WHEN '106' THEN 'icon.kanban'
    WHEN '107' THEN 'icon.user'
    ELSE icon
END
WHERE menu_id IN ('103', '104', '105', '106', '107');
