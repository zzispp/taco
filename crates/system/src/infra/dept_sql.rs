use super::dept::COLUMNS;

pub(super) fn insert_sql() -> &'static str {
    "INSERT INTO sys_dept (dept_id,parent_id,ancestors,dept_name,order_num,leader,phone,email,status,create_time) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)"
}

pub(super) fn update_sql() -> &'static str {
    "UPDATE sys_dept SET parent_id=$2,ancestors=$3,dept_name=$4,order_num=$5,leader=$6,phone=$7,email=$8,status=$9,update_time=CURRENT_TIMESTAMP WHERE dept_id=$1 AND del_flag='0'"
}

pub(super) fn list_sql() -> String {
    format!(
        "SELECT {COLUMNS} FROM sys_dept WHERE {} ORDER BY parent_id ASC, order_num ASC, dept_id ASC",
        predicate()
    )
}

pub(super) fn scoped_list_sql() -> String {
    format!(
        "SELECT {COLUMNS} FROM sys_dept d WHERE {} AND {} ORDER BY parent_id ASC, order_num ASC, dept_id ASC",
        scoped_filter_predicate(),
        scoped_predicate()
    )
}

fn predicate() -> &'static str {
    "del_flag='0' AND ($1::text IS NULL OR dept_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR leader ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR phone ILIKE '%' || $3 || '%') AND ($4::text IS NULL OR email ILIKE '%' || $4 || '%') AND ($5::text IS NULL OR status=$5) AND ($6::timestamptz IS NULL OR create_time >= $6) AND ($7::timestamptz IS NULL OR create_time <= $7)"
}

fn scoped_filter_predicate() -> &'static str {
    "d.del_flag='0' AND ($1::text IS NULL OR d.dept_name ILIKE '%' || $1 || '%') AND ($2::text IS NULL OR d.leader ILIKE '%' || $2 || '%') AND ($3::text IS NULL OR d.phone ILIKE '%' || $3 || '%') AND ($4::text IS NULL OR d.email ILIKE '%' || $4 || '%') AND ($5::text IS NULL OR d.status=$5) AND ($6::timestamptz IS NULL OR d.create_time >= $6) AND ($7::timestamptz IS NULL OR d.create_time <= $7)"
}

fn scoped_predicate() -> &'static str {
    "($8='1' OR ($8='2' AND d.dept_id = ANY($10)) OR ($8='3' AND $9::text IS NOT NULL AND d.dept_id=$9) OR ($8='4' AND $9::text IS NOT NULL AND (d.dept_id=$9 OR (',' || d.ancestors || ',') LIKE '%,' || $9 || ',%')) OR ($8='5' AND $9::text IS NOT NULL AND d.dept_id=$9))"
}

#[cfg(test)]
mod tests {
    use super::{list_sql, scoped_list_sql};

    #[test]
    fn dept_text_filters_use_case_insensitive_search() {
        for sql in [list_sql(), scoped_list_sql()] {
            assert!(sql.contains("dept_name ILIKE"));
            assert!(sql.contains("leader ILIKE"));
            assert!(sql.contains("phone ILIKE"));
            assert!(sql.contains("email ILIKE"));
        }
    }

    #[test]
    fn dept_time_filters_compare_timestamps_without_date_truncation() {
        for sql in [list_sql(), scoped_list_sql()] {
            assert!(sql.contains("create_time >="));
            assert!(sql.contains("create_time <="));
            assert!(!sql.contains("::date"));
        }
    }
}
