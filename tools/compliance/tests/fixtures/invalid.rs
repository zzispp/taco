pub fn invalid(first: bool, second: bool, third: bool, fourth: bool) -> Option<u8> {
    let mut value = Some(1);
    if first {
        if second {
            if third {
                if fourth {
                    value = None;
                }
            }
        }
    }
    value.unwrap();
    value
}
