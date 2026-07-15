#[test]
fn scheduled_task_macro_contracts_compile_as_declared() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/trybuild/pass/*.rs");
    tests.compile_fail("tests/trybuild/fail/*.rs");
}
