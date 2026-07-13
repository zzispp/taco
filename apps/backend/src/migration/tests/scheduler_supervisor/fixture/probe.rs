use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use scheduler::application::SchedulerTelemetry;

#[derive(Default)]
pub(crate) struct RuntimeProbe {
    pub(crate) leader: AtomicBool,
    pub(crate) leadership_acquisitions: AtomicUsize,
    pub(crate) timer_reconciles: AtomicUsize,
    pub(crate) notification_reconciles: AtomicUsize,
}

impl SchedulerTelemetry for RuntimeProbe {
    fn leadership(&self, leader: bool) {
        self.leader.store(leader, Ordering::SeqCst);
        if leader {
            self.leadership_acquisitions.fetch_add(1, Ordering::SeqCst);
        }
    }

    fn reconcile(&self, reason: &'static str, _success: bool) {
        match reason {
            "timer" => self.timer_reconciles.fetch_add(1, Ordering::SeqCst),
            "notification" => self.notification_reconciles.fetch_add(1, Ordering::SeqCst),
            _ => 0,
        };
    }

    fn runtime_error(&self, _operation: &'static str) {}
    fn execution(&self, _trigger: &'static str, _outcome: &'static str) {}
    fn active_executions(&self, _pending: usize, _running: usize) {}
    fn schedule_lag(&self, _seconds: f64) {}
}
