#[test]
fn scheduled_task_macro_pass_contracts_compile_as_declared() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/trybuild/pass/*.rs");
}

#[test]
fn scheduled_task_macro_fail_contracts_reject_invalid_declarations() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/fail/*.rs");
}
