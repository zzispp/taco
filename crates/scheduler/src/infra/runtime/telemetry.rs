use metrics::{counter, gauge, histogram};

use crate::application::SchedulerTelemetry;

const LEADER: &str = "scheduler_leader";
const RECONCILE: &str = "scheduler_reconcile_total";
const RUNTIME_ERRORS: &str = "scheduler_runtime_errors_total";
const EXECUTIONS: &str = "scheduler_executions_total";
const ACTIVE_EXECUTIONS: &str = "scheduler_active_executions";
const SCHEDULE_LAG: &str = "scheduler_schedule_lag_seconds";

#[derive(Clone, Copy, Default)]
pub struct MetricsSchedulerTelemetry;

impl SchedulerTelemetry for MetricsSchedulerTelemetry {
    fn leadership(&self, leader: bool) {
        gauge!(LEADER).set(if leader { 1.0 } else { 0.0 });
    }

    fn reconcile(&self, reason: &'static str, success: bool) {
        counter!(RECONCILE, "reason" => reason, "status" => status(success)).increment(1);
    }

    fn runtime_error(&self, operation: &'static str) {
        counter!(RUNTIME_ERRORS, "operation" => operation).increment(1);
    }

    fn execution(&self, trigger: &'static str, outcome: &'static str) {
        counter!(EXECUTIONS, "trigger" => trigger, "outcome" => outcome).increment(1);
    }

    fn active_executions(&self, pending: usize, running: usize) {
        gauge!(ACTIVE_EXECUTIONS, "state" => "pending").set(pending as f64);
        gauge!(ACTIVE_EXECUTIONS, "state" => "running").set(running as f64);
    }

    fn schedule_lag(&self, seconds: f64) {
        histogram!(SCHEDULE_LAG).record(seconds);
    }
}

const fn status(success: bool) -> &'static str {
    if success { "ok" } else { "error" }
}
