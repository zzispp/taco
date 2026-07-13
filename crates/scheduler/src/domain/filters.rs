use chrono::{DateTime, Utc};

use super::{ExecutionOutcome, JobStatus, TriggerType};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobListFilter {
    pub name: Option<String>,
    pub group: Option<String>,
    pub status: Option<JobStatus>,
    pub begin_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct JobLogListFilter {
    pub name: Option<String>,
    pub group: Option<String>,
    pub outcome: Option<ExecutionOutcome>,
    pub trigger: Option<TriggerType>,
    pub begin_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}
