use kernel::pagination::{CursorDirection, CursorPageRequest};

use crate::error::keys;
use crate::{FileError, FileResult};

use super::*;

const LIMIT: u64 = 2;
const FINGERPRINT: &str = "file-query";

#[test]
fn cursor_round_trip_preserves_navigation_direction() {
    let codec = FileCursorCodec::new(FINGERPRINT, LIMIT);
    let token = codec.encode(CursorDirection::Previous, &CursorBoundary::new("alpha", "entry-1"));

    let decoded = codec.decode(&token).unwrap();

    assert_eq!(decoded.sort_value, "alpha");
    assert_eq!(decoded.id, "entry-1");
    assert_eq!(decoded.direction, CursorDirection::Previous);
    assert_eq!(decoded.fingerprint, FINGERPRINT);
    assert_eq!(decoded.limit, LIMIT);
}

#[test]
fn cursor_rejects_changed_filter_or_limit() {
    let token = FileCursorCodec::new(FINGERPRINT, LIMIT).encode(CursorDirection::Next, &CursorBoundary::new("alpha", "entry-1"));

    assert_eq!(
        FileCursorCodec::new("different-query", LIMIT).decode(&token),
        Err(FileError::InvalidInput(keys::CURSOR_QUERY_MISMATCH))
    );
    assert_eq!(
        FileCursorCodec::new(FINGERPRINT, LIMIT + 1).decode(&token),
        Err(FileError::InvalidInput(keys::CURSOR_QUERY_MISMATCH))
    );
}

#[test]
fn first_page_only_exposes_the_next_boundary() {
    let context = context(None);

    let page = context.build_page(vec!["alpha", "beta", "gamma"], string_boundary).unwrap();
    let next = FileCursorCodec::new(FINGERPRINT, LIMIT)
        .decode(page.next_cursor.as_deref().expect("first page must have a next cursor"))
        .unwrap();

    assert_eq!(page.records, vec!["alpha", "beta"]);
    assert_eq!(next.direction, CursorDirection::Next);
    assert_eq!(next.sort_value, "beta");
    assert_eq!(next.id, "beta");
    assert_eq!(page.previous_cursor, None);
}

#[test]
fn previous_page_restores_logical_order_and_forward_navigation() {
    let token = FileCursorCodec::new(FINGERPRINT, LIMIT).encode(CursorDirection::Previous, &CursorBoundary::new("gamma", "gamma"));
    let context = context(Some(&token));

    let page = context.build_page(vec!["beta", "alpha"], string_boundary).unwrap();
    let next = FileCursorCodec::new(FINGERPRINT, LIMIT)
        .decode(page.next_cursor.as_deref().expect("a previous query must return to its source page"))
        .unwrap();

    assert_eq!(page.records, vec!["alpha", "beta"]);
    assert_eq!(next.direction, CursorDirection::Next);
    assert_eq!(next.sort_value, "beta");
    assert_eq!(page.previous_cursor, None);
}

#[test]
fn empty_forward_page_exposes_the_opposite_recovery_cursor() {
    let token = FileCursorCodec::new(FINGERPRINT, LIMIT).encode(CursorDirection::Next, &CursorBoundary::new("beta", "beta"));
    let context = context(Some(&token));

    let page = context.build_page::<&str, _>(Vec::new(), string_boundary).unwrap();
    let previous = FileCursorCodec::new(FINGERPRINT, LIMIT)
        .decode(page.previous_cursor.as_deref().expect("empty forward page must remain reversible"))
        .unwrap();

    assert_eq!(page.records, Vec::<&str>::new());
    assert_eq!(page.next_cursor, None);
    assert_eq!(previous.direction, CursorDirection::Previous);
    assert_eq!(previous.sort_value, "beta");
    assert_eq!(previous.id, "beta");
}

fn context(cursor: Option<&str>) -> FilePageContext<'static> {
    FilePageContext::new(
        cursor,
        FINGERPRINT,
        &CursorPageRequest {
            limit: LIMIT,
            cursor: cursor.map(str::to_owned),
        },
    )
    .unwrap()
}

fn string_boundary(value: &&str) -> FileResult<CursorBoundary> {
    Ok(CursorBoundary::new(*value, *value))
}
