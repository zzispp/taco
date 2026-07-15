pub(super) const LOCATION_MAX_CHARS: usize = 255;

pub(super) fn truncate(value: &str, max_chars: usize) -> String {
    let Some((boundary, _)) = value.char_indices().nth(max_chars) else {
        return value.into();
    };
    value[..boundary].into()
}
