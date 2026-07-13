use super::{data_page_sql, type_page_sql};

#[test]
fn dict_text_filters_use_case_insensitive_search() {
    let type_sql = type_page_sql();
    let data_sql = data_page_sql();

    assert!(type_sql.contains("dict_name ILIKE"));
    assert!(type_sql.contains("dict_type ILIKE"));
    assert!(data_sql.contains("dict_label ILIKE"));
}

#[test]
fn dict_type_time_filters_compare_timestamps_without_date_truncation() {
    for sql in [type_page_sql(), data_page_sql()] {
        assert!(sql.contains("create_time >="));
        assert!(sql.contains("create_time <="));
        assert!(!sql.contains("::date"));
    }
}
