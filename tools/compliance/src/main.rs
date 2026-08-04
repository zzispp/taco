use std::{env, process::ExitCode};

use compliance::{report::print_violations, rust::scan_workspace};

fn main() -> ExitCode {
    let root = match env::current_dir() {
        Ok(root) => root,
        Err(error) => {
            eprintln!("compliance: cannot resolve the current directory: {error}");
            return ExitCode::FAILURE;
        }
    };
    let violations = scan_workspace(&root);
    if violations.is_empty() {
        return ExitCode::SUCCESS;
    }
    print_violations(&violations);
    ExitCode::FAILURE
}
