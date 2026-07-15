use super::sql::USER_COLUMNS;

const FILTERED_USERS_TEMPLATE: &str = r#"
    SELECT {columns}
    FROM sys_user u
    WHERE u.del_flag = '0'
      AND ($1::text IS NULL OR u.user_name ILIKE '%' || $1 || '%')
      AND ($2::text IS NULL OR u.phonenumber ILIKE '%' || $2 || '%')
      AND ($3::text IS NULL OR u.status = $3)
      AND (
          $4::text IS NULL OR u.dept_id = $4 OR EXISTS (
              SELECT 1 FROM sys_dept d WHERE d.dept_id = u.dept_id AND d.del_flag = '0'
                AND (',' || d.ancestors || ',') LIKE '%,' || $4 || ',%'
          )
      )
      AND ($5::timestamptz IS NULL OR u.create_time >= $5)
      AND ($6::timestamptz IS NULL OR u.create_time <= $6)
      AND ($7::text IS NULL OR u.nick_name ILIKE '%' || $7 || '%')
      AND (
          $8::text IS NULL OR EXISTS (
              SELECT 1 FROM sys_dept d WHERE d.dept_id = u.dept_id AND d.del_flag = '0'
                AND d.dept_name ILIKE '%' || $8 || '%'
          )
      )
      AND ($9::text IS NULL OR u.email ILIKE '%' || $9 || '%')
      AND ($10::text IS NULL OR u.sex = $10)
      AND (
          cardinality($11::text[]) = 0 OR EXISTS (
              SELECT 1 FROM sys_user_post up WHERE up.user_id = u.user_id AND up.post_id::text = ANY($11::text[])
          )
      )
      AND (
          cardinality($12::text[]) = 0 OR EXISTS (
              SELECT 1 FROM sys_user_role ur WHERE ur.user_id = u.user_id AND ur.role_id::text = ANY($12::text[])
          )
      )
    {suffix}
"#;

const SCOPED_USERS_TEMPLATE: &str = r#"
    SELECT {columns}
    FROM sys_user u
    WHERE u.del_flag = '0'
      AND (
          $1 = '1'
          OR ($1 = '2' AND u.dept_id = ANY($4))
          OR ($1 = '3' AND $3 IS NOT NULL AND u.dept_id = $3)
          OR (
              $1 = '4' AND $3 IS NOT NULL AND EXISTS (
                  SELECT 1 FROM sys_dept d WHERE d.dept_id = u.dept_id AND d.del_flag = '0'
                    AND (d.dept_id = $3 OR (',' || d.ancestors || ',') LIKE '%,' || $3 || ',%')
              )
          )
          OR ($1 = '5' AND u.user_id = $2)
      )
      AND ($5::text IS NULL OR u.user_name ILIKE '%' || $5 || '%')
      AND ($6::text IS NULL OR u.phonenumber ILIKE '%' || $6 || '%')
      AND ($7::text IS NULL OR u.status = $7)
      AND (
          $8::text IS NULL OR u.dept_id = $8 OR EXISTS (
              SELECT 1 FROM sys_dept d WHERE d.dept_id = u.dept_id AND d.del_flag = '0'
                AND (',' || d.ancestors || ',') LIKE '%,' || $8 || ',%'
          )
      )
      AND ($9::timestamptz IS NULL OR u.create_time >= $9)
      AND ($10::timestamptz IS NULL OR u.create_time <= $10)
      AND ($11::text IS NULL OR u.nick_name ILIKE '%' || $11 || '%')
      AND (
          $12::text IS NULL OR EXISTS (
              SELECT 1 FROM sys_dept d WHERE d.dept_id = u.dept_id AND d.del_flag = '0'
                AND d.dept_name ILIKE '%' || $12 || '%'
          )
      )
      AND ($13::text IS NULL OR u.email ILIKE '%' || $13 || '%')
      AND ($14::text IS NULL OR u.sex = $14)
      AND (
          cardinality($15::text[]) = 0 OR EXISTS (
              SELECT 1 FROM sys_user_post up WHERE up.user_id = u.user_id AND up.post_id::text = ANY($15::text[])
          )
      )
      AND (
          cardinality($16::text[]) = 0 OR EXISTS (
              SELECT 1 FROM sys_user_role ur WHERE ur.user_id = u.user_id AND ur.role_id::text = ANY($16::text[])
          )
      )
    {suffix}
"#;

pub(in crate::infra::user_repository) fn filtered_users(suffix: &str) -> String {
    FILTERED_USERS_TEMPLATE.replace("{columns}", USER_COLUMNS).replace("{suffix}", suffix)
}

pub(in crate::infra::user_repository) fn scoped_users(suffix: &str) -> String {
    SCOPED_USERS_TEMPLATE.replace("{columns}", USER_COLUMNS).replace("{suffix}", suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_suffixes_follow_filter_bindings() {
        let users = filtered_users("AND (u.create_time,u.user_id) <= ($13,$14) LIMIT $17");
        let scoped = scoped_users("AND (u.create_time,u.user_id) <= ($17,$18) LIMIT $21");

        assert!(users.contains("$12::text[]"));
        assert!(users.contains("($13,$14)"));
        assert!(scoped.contains("$16::text[]"));
        assert!(scoped.contains("($17,$18)"));
    }

    #[test]
    fn filters_compare_typed_timestamps() {
        for sql in [filtered_users(""), scoped_users("")] {
            assert!(!sql.contains("create_time::date"));
            assert!(sql.contains("::timestamptz IS NULL"));
            assert!(sql.contains("u.create_time >="));
            assert!(sql.contains("u.create_time <="));
        }
    }
}
