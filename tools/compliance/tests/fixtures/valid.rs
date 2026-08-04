pub fn fallback(value: Option<u8>) -> u8 {
    value.unwrap_or_default()
}
