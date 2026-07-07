use crate::domain::{ServerDiskMetrics, ServerHealth, ServerHealthIssue, ServerHealthIssueKind, ServerHealthStatus};

const HEALTH_YELLOW_PERCENT: f32 = 85.0;
const HEALTH_RED_PERCENT: f32 = 95.0;
const CPU_TARGET: &str = "total";
const MEMORY_TARGET: &str = "physical";

pub fn evaluate_dashboard_health(cpu_usage: f32, memory_usage: f32, disks: &[ServerDiskMetrics]) -> ServerHealth {
    let mut issues = Vec::new();
    push_issue(&mut issues, HealthSignal::new(ServerHealthIssueKind::Cpu, CPU_TARGET, cpu_usage));
    push_issue(&mut issues, HealthSignal::new(ServerHealthIssueKind::Memory, MEMORY_TARGET, memory_usage));
    for disk in disks {
        push_issue(
            &mut issues,
            HealthSignal::new(ServerHealthIssueKind::Disk, &disk.mount_point, disk.usage_percent),
        );
    }
    ServerHealth {
        status: aggregate_status(&issues),
        issues,
    }
}

struct HealthSignal<'a> {
    kind: ServerHealthIssueKind,
    target: &'a str,
    usage_percent: f32,
}

impl<'a> HealthSignal<'a> {
    const fn new(kind: ServerHealthIssueKind, target: &'a str, usage_percent: f32) -> Self {
        Self { kind, target, usage_percent }
    }
}

fn push_issue(issues: &mut Vec<ServerHealthIssue>, signal: HealthSignal<'_>) {
    let status = usage_status(signal.usage_percent);
    if status == ServerHealthStatus::Green {
        return;
    }
    issues.push(ServerHealthIssue {
        kind: signal.kind,
        target: signal.target.into(),
        usage_percent: signal.usage_percent,
        threshold_percent: threshold_for(&status),
        status,
    });
}

fn usage_status(usage_percent: f32) -> ServerHealthStatus {
    if usage_percent >= HEALTH_RED_PERCENT {
        return ServerHealthStatus::Red;
    }
    if usage_percent >= HEALTH_YELLOW_PERCENT {
        return ServerHealthStatus::Yellow;
    }
    ServerHealthStatus::Green
}

fn threshold_for(status: &ServerHealthStatus) -> f32 {
    match status {
        ServerHealthStatus::Red => HEALTH_RED_PERCENT,
        ServerHealthStatus::Yellow => HEALTH_YELLOW_PERCENT,
        ServerHealthStatus::Green => 0.0,
    }
}

fn aggregate_status(issues: &[ServerHealthIssue]) -> ServerHealthStatus {
    if issues.iter().any(|issue| issue.status == ServerHealthStatus::Red) {
        return ServerHealthStatus::Red;
    }
    if issues.iter().any(|issue| issue.status == ServerHealthStatus::Yellow) {
        return ServerHealthStatus::Yellow;
    }
    ServerHealthStatus::Green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_is_green_below_thresholds() {
        let health = evaluate_dashboard_health(10.0, 20.0, &[disk("/", 30.0)]);
        assert_eq!(health.status, ServerHealthStatus::Green);
        assert_eq!(health.issues, Vec::new());
    }

    #[test]
    fn health_is_yellow_at_yellow_threshold() {
        let health = evaluate_dashboard_health(HEALTH_YELLOW_PERCENT, 20.0, &[]);
        assert_eq!(health.status, ServerHealthStatus::Yellow);
        assert_eq!(health.issues[0].threshold_percent, HEALTH_YELLOW_PERCENT);
    }

    #[test]
    fn health_is_red_at_red_threshold() {
        let health = evaluate_dashboard_health(10.0, 20.0, &[disk("/data", HEALTH_RED_PERCENT)]);
        assert_eq!(health.status, ServerHealthStatus::Red);
        assert_eq!(health.issues[0].target, "/data");
    }

    fn disk(mount_point: &str, usage_percent: f32) -> ServerDiskMetrics {
        ServerDiskMetrics {
            name: "disk".into(),
            mount_point: mount_point.into(),
            file_system: "ext4".into(),
            total_bytes: 100,
            available_bytes: 100,
            used_bytes: 0,
            usage_percent,
        }
    }
}
