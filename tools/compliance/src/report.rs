use std::{fmt, path::PathBuf};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Violation {
    pub path: PathBuf,
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
}

pub struct ViolationDetails {
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
}

impl Violation {
    pub fn new(path: PathBuf, details: ViolationDetails) -> Self {
        Self {
            path,
            line: details.line,
            rule: details.rule,
            message: details.message,
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}: {}: {}", self.path.display(), self.line, self.rule, self.message)
    }
}

pub fn print_violations(violations: &[Violation]) {
    let mut ordered = violations.to_vec();
    ordered.sort();
    for violation in ordered {
        eprintln!("{violation}");
    }
}
