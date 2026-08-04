mod analysis;
mod metrics;
mod visitor;
mod workspace;

pub const MAX_FUNCTION_LINES: usize = 50;
pub const MAX_FILE_LINES: usize = 300;
pub const MAX_NESTING_DEPTH: usize = 3;
pub const MAX_POSITIONAL_PARAMETERS: usize = 3;
pub const MAX_CYCLOMATIC_COMPLEXITY: usize = 10;

pub use analysis::analyze_source;
pub use workspace::scan_workspace;
