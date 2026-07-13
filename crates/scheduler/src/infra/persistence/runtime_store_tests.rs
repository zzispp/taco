use chrono::{DateTime, Utc};

use crate::{
    application::OccurrenceAction,
    domain::{ConcurrentPolicy, ExecutionOutcome, TriggerType},
};

use super::runtime_store::{OccurrenceDecision, SKIPPED_OVERLAP, occurrence_decision};

fn fixed_time() -> DateTime<Utc> {
    DateTime::from_timestamp(1_700_000_000, 0).expect("test timestamp must be valid")
}

#[test]
fn allow_policy_keeps_an_occurrence_pending_when_an_execution_is_active() {
    let (trigger, terminal) = occurrence_decision(OccurrenceDecision {
        action: &OccurrenceAction::Queue(TriggerType::Scheduled),
        concurrent: ConcurrentPolicy::Allow,
        has_active: true,
        now: fixed_time(),
    });

    assert_eq!(trigger, TriggerType::Scheduled);
    assert!(terminal.is_none());
}

#[test]
fn disallow_policy_materializes_one_skipped_overlap() {
    let (trigger, terminal) = occurrence_decision(OccurrenceDecision {
        action: &OccurrenceAction::Queue(TriggerType::Scheduled),
        concurrent: ConcurrentPolicy::Disallow,
        has_active: true,
        now: fixed_time(),
    });
    let terminal = terminal.expect("overlap must be terminal");

    assert_eq!(trigger, TriggerType::Scheduled);
    assert_eq!(terminal.outcome, ExecutionOutcome::Skipped);
    assert_eq!(terminal.message.key, SKIPPED_OVERLAP);
    assert_eq!(terminal.ended_at, fixed_time());
}
