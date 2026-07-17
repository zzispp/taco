CREATE TABLE sys_log_menu_hierarchy_role_grant (
    role_id VARCHAR(36) PRIMARY KEY REFERENCES sys_role(role_id) ON DELETE CASCADE
);

WITH inserted AS (
    INSERT INTO sys_role_menu (role_id, menu_id)
    SELECT child.role_id, '111'
    FROM sys_role_menu AS child
    WHERE child.menu_id = '109'
      AND NOT EXISTS (
          SELECT 1
          FROM sys_role_menu AS parent
          WHERE parent.role_id = child.role_id
            AND parent.menu_id = '111'
      )
    ON CONFLICT DO NOTHING
    RETURNING role_id
)
INSERT INTO sys_log_menu_hierarchy_role_grant (role_id)
SELECT role_id
FROM inserted;
