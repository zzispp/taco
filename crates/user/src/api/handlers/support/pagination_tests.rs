use super::*;

#[test]
fn export_page_completion_uses_loaded_rows_and_empty_page() {
    assert!(!is_last_export_page(10, 5, 20).unwrap());
    assert!(is_last_export_page(10, 10, 20).unwrap());
    assert!(is_last_export_page(10, 0, 20).unwrap());
}

#[test]
fn export_page_completion_rejects_loaded_row_count_overflow() {
    assert_page_overflow(is_last_export_page(usize::MAX, 1, u64::MAX));
}

#[test]
fn export_page_number_rejects_increment_overflow() {
    assert_page_overflow(next_export_page(u64::MAX));
}

#[test]
fn export_total_conversion_matches_the_platform_width() {
    #[cfg(target_pointer_width = "64")]
    assert_eq!(export_total(u64::MAX).unwrap(), usize::MAX);

    #[cfg(target_pointer_width = "32")]
    assert_page_overflow(export_total(u64::from(u32::MAX) + 1));
}

fn assert_page_overflow<T>(result: ApiResult<T>) {
    let Err(ApiError(AppError::InvalidInput(error))) = result else {
        panic!("expected pagination overflow error");
    };
    assert_eq!(error.key(), "errors.validation.page_overflow");
}
