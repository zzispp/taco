use time::OffsetDateTime;

use super::SystemLogLevel;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SystemLogFilter {
    pub keyword: Option<String>,
    pub levels: Vec<SystemLogLevel>,
    pub target: Option<String>,
    pub begin_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
}
