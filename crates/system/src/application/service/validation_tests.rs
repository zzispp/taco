use super::*;

#[test]
fn page_validation_rejects_excessive_size_and_overflow() {
    let excessive = validate_page(CursorPageRequest {
        limit: kernel::pagination::MAX_CURSOR_LIMIT + 1,
        cursor: None,
    });
    let below_minimum = validate_page(CursorPageRequest { limit: 0, cursor: None });

    assert!(matches!(excessive, Err(SystemError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"));
    assert!(matches!(below_minimum, Err(SystemError::InvalidInput(message)) if message.key() == "errors.validation.cursor_limit_range"));
}
