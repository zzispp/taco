pub fn make_callbacks() {
    let too_many_parameters = |first: bool, second: bool, third: bool, fourth: bool| {
        first || second || third || fourth
    };
    let too_complex = || {
        if true {}
        if true {}
        if true {}
        if true {}
        if true {}
        if true {}
        if true {}
        if true {}
        if true {}
        if true {}
    };
    let _ = (too_many_parameters, too_complex);
}
