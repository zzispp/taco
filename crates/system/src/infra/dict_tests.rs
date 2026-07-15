use kernel::pagination::CursorPageRequest;

use super::{DictDataListFilter, DictTypeListFilter, data_filtered_query, type_filtered_query};

#[test]
fn dict_text_filters_use_case_insensitive_search() {
    let type_sql = type_filtered_query(&type_filter()).into_string();
    let data_sql = data_filtered_query(&data_filter()).into_string();

    assert!(type_sql.contains("dict_name ILIKE"));
    assert!(type_sql.contains("dict_type ILIKE"));
    assert!(data_sql.contains("dict_label ILIKE"));
}

#[test]
fn dict_time_filters_use_native_timestamps_without_offset() {
    for sql in [
        type_filtered_query(&type_filter()).into_string(),
        data_filtered_query(&data_filter()).into_string(),
    ] {
        assert!(sql.contains("create_time>="));
        assert!(sql.contains("create_time<="));
        assert!(!sql.contains("::text"));
        assert!(!sql.contains("OFFSET"));
    }
}

fn type_filter() -> DictTypeListFilter {
    DictTypeListFilter {
        page: CursorPageRequest::default(),
        dict_name: Some("name".into()),
        dict_type: Some("type".into()),
        status: None,
        begin_time: Some(time::OffsetDateTime::UNIX_EPOCH),
        end_time: Some(time::OffsetDateTime::UNIX_EPOCH),
    }
}

fn data_filter() -> DictDataListFilter {
    DictDataListFilter {
        page: CursorPageRequest::default(),
        dict_type: Some("type".into()),
        dict_label: Some("label".into()),
        status: None,
        begin_time: Some(time::OffsetDateTime::UNIX_EPOCH),
        end_time: Some(time::OffsetDateTime::UNIX_EPOCH),
    }
}
