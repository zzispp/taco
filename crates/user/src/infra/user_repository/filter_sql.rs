use super::sql::USER_COLUMNS;

const FILTERED_USERS_TEMPLATE: &str = r#"
    SELECT {columns}
    FROM sys_user u
    WHERE u.del_flag = '0'
      AND ($1::text IS NULL OR u.user_name ILIKE '%' || $1 || '%')
      AND ($2::text IS NULL OR u.phonenumber ILIKE '%' || $2 || '%')
      AND ($3::text IS NULL OR u.status = $3)
      AND (
          $4::text IS NULL
          OR u.dept_id = $4
          OR EXISTS (
              SELECT 1 FROM sys_dept d
              WHERE d.dept_id = u.dept_id
                AND d.del_flag = '0'
                AND (',' || d.ancestors || ',') LIKE '%,' || $4 || ',%'
          )
      )
      AND ($5::text IS NULL OR u.create_time::date >= $5::date)
      AND ($6::text IS NULL OR u.create_time::date <= $6::date)
      AND ($7::text IS NULL OR u.nick_name ILIKE '%' || $7 || '%')
      AND (
          $8::text IS NULL
          OR EXISTS (
              SELECT 1 FROM sys_dept d
              WHERE d.dept_id = u.dept_id AND d.del_flag = '0' AND d.dept_name ILIKE '%' || $8 || '%'
          )
      )
      AND ($9::text IS NULL OR u.email ILIKE '%' || $9 || '%')
      AND ($10::text IS NULL OR u.sex = $10)
      AND (
          cardinality($11::text[]) = 0
          OR EXISTS (SELECT 1 FROM sys_user_post up WHERE up.user_id = u.user_id AND up.post_id::text = ANY($11::text[]))
      )
      AND (
          cardinality($12::text[]) = 0
          OR EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id = u.user_id AND ur.role_id::text = ANY($12::text[]))
      )
    {suffix}
"#;

const FILTERED_USERS_TOTAL: &str = r#"
    SELECT COUNT(*)
    FROM sys_user u
    WHERE u.del_flag = '0'
      AND ($1::text IS NULL OR u.user_name ILIKE '%' || $1 || '%')
      AND ($2::text IS NULL OR u.phonenumber ILIKE '%' || $2 || '%')
      AND ($3::text IS NULL OR u.status = $3)
      AND (
          $4::text IS NULL
          OR u.dept_id = $4
          OR EXISTS (
              SELECT 1 FROM sys_dept d
              WHERE d.dept_id = u.dept_id
                AND d.del_flag = '0'
                AND (',' || d.ancestors || ',') LIKE '%,' || $4 || ',%'
          )
      )
      AND ($5::text IS NULL OR u.create_time::date >= $5::date)
      AND ($6::text IS NULL OR u.create_time::date <= $6::date)
      AND ($7::text IS NULL OR u.nick_name ILIKE '%' || $7 || '%')
      AND (
          $8::text IS NULL
          OR EXISTS (
              SELECT 1 FROM sys_dept d
              WHERE d.dept_id = u.dept_id AND d.del_flag = '0' AND d.dept_name ILIKE '%' || $8 || '%'
          )
      )
      AND ($9::text IS NULL OR u.email ILIKE '%' || $9 || '%')
      AND ($10::text IS NULL OR u.sex = $10)
      AND (
          cardinality($11::text[]) = 0
          OR EXISTS (SELECT 1 FROM sys_user_post up WHERE up.user_id = u.user_id AND up.post_id::text = ANY($11::text[]))
      )
      AND (
          cardinality($12::text[]) = 0
          OR EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id = u.user_id AND ur.role_id::text = ANY($12::text[]))
      )
"#;

const SCOPED_USER_IDS: &str = r#"
    SELECT u.user_id
    FROM sys_user u
    WHERE u.del_flag = '0' AND (
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
    AND ($5::text IS NULL OR u.user_name ILIKE '%' || $5 || '%')
    AND ($6::text IS NULL OR u.phonenumber ILIKE '%' || $6 || '%')
    AND ($7::text IS NULL OR u.status = $7)
    AND (
        $8::text IS NULL
        OR u.dept_id = $8
        OR EXISTS (
            SELECT 1 FROM sys_dept d
            WHERE d.dept_id = u.dept_id
              AND d.del_flag = '0'
              AND (',' || d.ancestors || ',') LIKE '%,' || $8 || ',%'
        )
    )
    AND ($9::text IS NULL OR u.create_time::date >= $9::date)
    AND ($10::text IS NULL OR u.create_time::date <= $10::date)
    AND ($11::text IS NULL OR u.nick_name ILIKE '%' || $11 || '%')
    AND (
        $12::text IS NULL
        OR EXISTS (
            SELECT 1 FROM sys_dept d
            WHERE d.dept_id = u.dept_id AND d.del_flag = '0' AND d.dept_name ILIKE '%' || $12 || '%'
        )
    )
    AND ($13::text IS NULL OR u.email ILIKE '%' || $13 || '%')
    AND ($14::text IS NULL OR u.sex = $14)
    AND (
        cardinality($15::text[]) = 0
        OR EXISTS (SELECT 1 FROM sys_user_post up WHERE up.user_id = u.user_id AND up.post_id::text = ANY($15::text[]))
    )
    AND (
        cardinality($16::text[]) = 0
        OR EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id = u.user_id AND ur.role_id::text = ANY($16::text[]))
    )
    ORDER BY u.create_time ASC
    LIMIT $17 OFFSET $18
"#;

const SCOPED_USER_TOTAL: &str = r#"
    SELECT COUNT(*)
    FROM sys_user u
    WHERE u.del_flag = '0' AND (
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
    AND ($5::text IS NULL OR u.user_name ILIKE '%' || $5 || '%')
    AND ($6::text IS NULL OR u.phonenumber ILIKE '%' || $6 || '%')
    AND ($7::text IS NULL OR u.status = $7)
    AND (
        $8::text IS NULL
        OR u.dept_id = $8
        OR EXISTS (
            SELECT 1 FROM sys_dept d
            WHERE d.dept_id = u.dept_id
              AND d.del_flag = '0'
              AND (',' || d.ancestors || ',') LIKE '%,' || $8 || ',%'
        )
    )
    AND ($9::text IS NULL OR u.create_time::date >= $9::date)
    AND ($10::text IS NULL OR u.create_time::date <= $10::date)
    AND ($11::text IS NULL OR u.nick_name ILIKE '%' || $11 || '%')
    AND (
        $12::text IS NULL
        OR EXISTS (
            SELECT 1 FROM sys_dept d
            WHERE d.dept_id = u.dept_id AND d.del_flag = '0' AND d.dept_name ILIKE '%' || $12 || '%'
        )
    )
    AND ($13::text IS NULL OR u.email ILIKE '%' || $13 || '%')
    AND ($14::text IS NULL OR u.sex = $14)
    AND (
        cardinality($15::text[]) = 0
        OR EXISTS (SELECT 1 FROM sys_user_post up WHERE up.user_id = u.user_id AND up.post_id::text = ANY($15::text[]))
    )
    AND (
        cardinality($16::text[]) = 0
        OR EXISTS (SELECT 1 FROM sys_user_role ur WHERE ur.user_id = u.user_id AND ur.role_id::text = ANY($16::text[]))
    )
"#;

pub(in crate::infra::user_repository) fn filtered_users(suffix: &str) -> String {
    FILTERED_USERS_TEMPLATE.replace("{columns}", USER_COLUMNS).replace("{suffix}", suffix)
}

pub(in crate::infra::user_repository) const fn filtered_users_total() -> &'static str {
    FILTERED_USERS_TOTAL
}

pub(in crate::infra::user_repository) const fn scoped_user_ids() -> &'static str {
    SCOPED_USER_IDS
}

pub(in crate::infra::user_repository) const fn scoped_user_total() -> &'static str {
    SCOPED_USER_TOTAL
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_user_queries_bind_filter_placeholders_to_actual_bind_order() {
        let ids = scoped_user_ids();
        let total = scoped_user_total();

        for sql in [ids, total] {
            assert!(sql.contains("$5::text IS NULL OR u.user_name ILIKE"));
            assert!(sql.contains("$6::text IS NULL OR u.phonenumber ILIKE"));
            assert!(sql.contains("$7::text IS NULL OR u.status = $7"));
            assert!(sql.contains("$8::text IS NULL\n        OR u.dept_id = $8"));
            assert!(sql.contains("$11::text IS NULL OR u.nick_name ILIKE"));
            assert!(sql.contains("$13::text IS NULL OR u.email ILIKE"));
            assert!(sql.contains("cardinality($15::text[]) = 0"));
            assert!(sql.contains("cardinality($16::text[]) = 0"));
        }
        assert!(ids.contains("LIMIT $17 OFFSET $18"));
    }

    #[test]
    fn filtered_users_slice_uses_extra_filter_placeholders_before_pagination() {
        let sql = filtered_users("ORDER BY u.create_time ASC LIMIT $13 OFFSET $14");

        assert!(sql.contains("$1::text IS NULL OR u.user_name ILIKE"));
        assert!(sql.contains("$2::text IS NULL OR u.phonenumber ILIKE"));
        assert!(sql.contains("$7::text IS NULL OR u.nick_name ILIKE"));
        assert!(sql.contains("$9::text IS NULL OR u.email ILIKE"));
        assert!(sql.contains("$10::text IS NULL OR u.sex = $10"));
        assert!(sql.contains("cardinality($11::text[]) = 0"));
        assert!(sql.contains("cardinality($12::text[]) = 0"));
        assert!(sql.contains("LIMIT $13 OFFSET $14"));
    }
}
