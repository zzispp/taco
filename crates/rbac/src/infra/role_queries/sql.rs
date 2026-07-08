use kernel::pagination::Page;
use storage::{StorageResult, database::to_u64};

use crate::{application::RoleListFilter, domain::Role};

use super::{RoleRecord, role};

pub(super) const ROLE_COLUMNS: &str = r#"
    r.role_id, r.role_name, r.role_key, r.role_sort, r.data_scope, r.menu_check_strictly,
    r.dept_check_strictly, r.status, r.system, r.remark, r.create_time::text AS create_time
"#;

pub(super) fn insert_role_sql() -> &'static str {
    "INSERT INTO sys_role (role_id, role_name, role_key, role_sort, data_scope, menu_check_strictly, dept_check_strictly, status, del_flag, system, remark, create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'0',FALSE,$9,$10)"
}

pub(super) fn update_role_sql() -> &'static str {
    "UPDATE sys_role SET role_name=$2, role_key=$3, role_sort=$4, data_scope=$5, menu_check_strictly=$6, dept_check_strictly=$7, status=$8, remark=$9, update_time=CURRENT_TIMESTAMP WHERE role_id=$1 AND del_flag='0'"
}

pub(super) fn permission_query() -> &'static str {
    r#"
    SELECT r.role_key, r.status, r.data_scope, m.perms
    FROM sys_role r
    CROSS JOIN sys_menu m
    WHERE r.role_key = 'admin' AND r.del_flag = '0'
    UNION
    SELECT r.role_key, r.status, r.data_scope, m.perms
    FROM sys_role r
    LEFT JOIN sys_role_menu rm ON rm.role_id = r.role_id
    LEFT JOIN sys_menu m ON m.menu_id = rm.menu_id
    WHERE r.role_key <> 'admin' AND r.del_flag = '0'
    "#
}

pub(super) fn dept_query() -> &'static str {
    "SELECT r.role_key, rd.dept_id FROM sys_role r INNER JOIN sys_role_dept rd ON rd.role_id = r.role_id WHERE r.del_flag = '0'"
}

pub(super) fn role_page(items: Vec<RoleRecord>, total: i64, filter: RoleListFilter) -> StorageResult<Page<Role>> {
    Ok(Page {
        items: items.into_iter().map(role).collect(),
        total: to_u64(total)?,
        page: filter.page.page,
        page_size: filter.page.page_size,
    })
}

pub(super) fn role_page_sql() -> String {
    format!(
        "SELECT {ROLE_COLUMNS} FROM sys_role r WHERE {} ORDER BY r.role_sort ASC LIMIT $6 OFFSET $7",
        role_where()
    )
}

pub(super) fn role_total_sql() -> String {
    format!("SELECT COUNT(*) FROM sys_role r WHERE {}", role_where())
}

pub(super) fn role_scoped_page_sql() -> String {
    format!(
        "SELECT DISTINCT {ROLE_COLUMNS} FROM sys_role r LEFT JOIN sys_user_role ur ON ur.role_id=r.role_id LEFT JOIN sys_user u ON u.user_id=ur.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE {} AND {} ORDER BY r.role_sort ASC LIMIT $10 OFFSET $11",
        role_where(),
        role_scope_where()
    )
}

pub(super) fn role_scoped_total_sql() -> String {
    format!(
        "SELECT COUNT(DISTINCT r.role_id) FROM sys_role r LEFT JOIN sys_user_role ur ON ur.role_id=r.role_id LEFT JOIN sys_user u ON u.user_id=ur.user_id LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE {} AND {}",
        role_where(),
        role_scope_where()
    )
}

pub(super) fn role_users_page_sql(scoped: bool) -> String {
    format!(
        "SELECT u.user_id,u.user_name AS username,u.nick_name,u.dept_id,u.phonenumber,u.email,u.status {} ORDER BY u.create_time ASC LIMIT $9 OFFSET $10",
        role_users_base(scoped)
    )
}

pub(super) fn role_users_total_sql(scoped: bool) -> String {
    format!("SELECT COUNT(*) {}", role_users_base(scoped))
}

pub(super) fn scoped_user_ids_sql() -> &'static str {
    "SELECT u.user_id FROM sys_user u LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE u.del_flag='0' AND u.user_id = ANY($1) AND ($2='1' OR ($2='2' AND u.dept_id = ANY($5)) OR ($2='3' AND $4::text IS NOT NULL AND u.dept_id=$4) OR ($2='4' AND $4::text IS NOT NULL AND (u.dept_id=$4 OR (',' || d.ancestors || ',') LIKE '%,' || $4 || ',%')) OR ($2='5' AND u.user_id=$3))"
}

fn role_where() -> &'static str {
    "r.del_flag='0' AND ($1::text IS NULL OR r.role_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR r.role_key ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR r.status=$3) AND ($4::text IS NULL OR r.create_time::date >= $4::date) AND ($5::text IS NULL OR r.create_time::date <= $5::date)"
}

fn role_scope_where() -> &'static str {
    "($6='1' OR ($6='2' AND u.dept_id = ANY($9)) OR ($6='3' AND $8::text IS NOT NULL AND u.dept_id=$8) OR ($6='4' AND $8::text IS NOT NULL AND (u.dept_id=$8 OR (',' || d.ancestors || ',') LIKE '%,' || $8 || ',%')) OR ($6='5' AND u.user_id=$7))"
}

fn role_users_base(scoped: bool) -> String {
    let scope = if scoped { format!(" AND {}", user_scope_where()) } else { String::new() };
    format!(
        "FROM sys_user u LEFT JOIN sys_dept d ON d.dept_id=u.dept_id WHERE u.del_flag='0' AND ($2::text IS NULL OR u.user_name ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR u.phonenumber ILIKE '%' || $3 || '%') AND (($4 AND EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id=u.user_id AND ur.role_id=$1)) OR (NOT $4 AND NOT EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id=u.user_id AND ur.role_id=$1))){}",
        scope
    )
}

fn user_scope_where() -> &'static str {
    "($5='1' OR ($5='2' AND u.dept_id = ANY($8)) OR ($5='3' AND $7::text IS NOT NULL AND u.dept_id=$7) OR ($5='4' AND $7::text IS NOT NULL AND (u.dept_id=$7 OR (',' || d.ancestors || ',') LIKE '%,' || $7 || ',%')) OR ($5='5' AND u.user_id=$6))"
}
