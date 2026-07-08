pub const USER_COLUMNS: &str = r#"
    user_id, dept_id, user_name, nick_name, email, phonenumber, sex, avatar,
    password, status, auth_source, email_verified, remark, create_time::text AS create_time
"#;

pub fn insert_user() -> &'static str {
    "INSERT INTO sys_user (user_id, dept_id, user_name, nick_name, email, phonenumber, sex, password, status, remark, create_time, pwd_update_date) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$11)"
}

pub fn update_with_password() -> &'static str {
    r#"
    UPDATE sys_user
    SET dept_id=$2,user_name=$3,nick_name=$4,email=$5,phonenumber=$6,sex=$7,status=$8,remark=$9,
        password=$10,pwd_update_date=CURRENT_TIMESTAMP,update_time=CURRENT_TIMESTAMP
    WHERE user_id=$1 AND del_flag='0'
    "#
}

pub fn update_without_password() -> &'static str {
    r#"
    UPDATE sys_user
    SET dept_id=$2,user_name=$3,nick_name=$4,email=$5,phonenumber=$6,sex=$7,status=$8,remark=$9,update_time=CURRENT_TIMESTAMP
    WHERE user_id=$1 AND del_flag='0'
    "#
}

pub fn role_query() -> &'static str {
    "SELECT r.role_id, r.role_name, r.role_key FROM sys_role r INNER JOIN sys_user_role ur ON ur.role_id = r.role_id WHERE ur.user_id = $1 AND r.del_flag = '0' ORDER BY r.role_sort ASC"
}

pub fn role_group_query() -> &'static str {
    r#"
    SELECT COALESCE(string_agg(r.role_name, ',' ORDER BY r.role_sort ASC), '')
    FROM sys_role r
    INNER JOIN sys_user_role ur ON ur.role_id = r.role_id
    WHERE ur.user_id = $1 AND r.del_flag = '0'
    "#
}

pub fn post_group_query() -> &'static str {
    r#"
    SELECT COALESCE(string_agg(p.post_name, ',' ORDER BY p.post_sort ASC), '')
    FROM sys_post p
    INNER JOIN sys_user_post up ON up.post_id = p.post_id
    WHERE up.user_id = $1
    "#
}

pub fn dept_name_query() -> &'static str {
    r#"
    SELECT d.dept_name
    FROM sys_user u
    INNER JOIN sys_dept d ON d.dept_id = u.dept_id
    WHERE u.user_id = $1 AND u.del_flag = '0' AND d.del_flag = '0'
    "#
}

pub fn permission_query() -> &'static str {
    r#"
    SELECT DISTINCT m.perms
    FROM sys_menu m
    CROSS JOIN sys_user_role ur
    INNER JOIN sys_role r ON r.role_id = ur.role_id
    WHERE ur.user_id = $1 AND r.role_key = 'admin' AND r.status = '0' AND r.del_flag = '0'
      AND m.status = '0' AND m.perms IS NOT NULL AND m.perms <> ''
    UNION
    SELECT DISTINCT m.perms
    FROM sys_menu m
    INNER JOIN sys_role_menu rm ON rm.menu_id = m.menu_id
    INNER JOIN sys_user_role ur ON ur.role_id = rm.role_id
    INNER JOIN sys_role r ON r.role_id = ur.role_id
    WHERE ur.user_id = $1 AND r.role_key <> 'admin' AND r.status = '0' AND r.del_flag = '0'
      AND m.status = '0' AND m.perms IS NOT NULL AND m.perms <> ''
    ORDER BY perms ASC
    "#
}

pub fn scoped_existing_user_ids() -> &'static str {
    r#"
    SELECT u.user_id
    FROM sys_user u
    WHERE u.del_flag = '0'
      AND u.user_id = ANY($5)
      AND (
        $1 = '1'
        OR ($1 = '2' AND u.dept_id = ANY($4))
        OR ($1 = '3' AND $3 IS NOT NULL AND u.dept_id = $3)
        OR (
            $1 = '4' AND $3 IS NOT NULL AND EXISTS (
                SELECT 1 FROM sys_dept d
                WHERE d.dept_id = u.dept_id
                  AND d.del_flag = '0'
                  AND (d.dept_id = $3 OR (',' || d.ancestors || ',') LIKE '%,' || $3 || ',%')
            )
        )
        OR ($1 = '5' AND u.user_id = $2)
    )
    ORDER BY u.create_time ASC
    "#
}
