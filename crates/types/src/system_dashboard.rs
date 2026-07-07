use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerDashboard {
    pub host: ServerHostInfo,
    pub cpu: ServerCpuMetrics,
    pub memory: ServerMemoryMetrics,
    pub disks: Vec<ServerDiskMetrics>,
    pub network: ServerNetworkSummary,
    pub top_processes: Vec<ServerProcessMetrics>,
    pub health: ServerHealth,
    pub sampled_at: String,
    pub sample_duration_millis: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerHostInfo {
    pub hostname: Option<String>,
    pub public_ips: Vec<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,
    pub cpu_arch: String,
    pub cpu_brand: Option<String>,
    pub physical_core_count: Option<usize>,
    pub logical_core_count: usize,
    pub uptime_seconds: u64,
    pub boot_time_unix_seconds: u64,
    pub total_memory_bytes: u64,
    pub total_disk_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerCpuMetrics {
    pub total_usage_percent: f32,
    pub load_average: ServerLoadAverage,
    pub cores: Vec<ServerCpuCoreMetrics>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerLoadAverage {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerCpuCoreMetrics {
    pub name: String,
    pub usage_percent: f32,
    pub frequency_mhz: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerMemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub available_bytes: u64,
    pub usage_percent: f32,
    pub total_swap_bytes: u64,
    pub used_swap_bytes: u64,
    pub free_swap_bytes: u64,
    pub swap_usage_percent: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerDiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub used_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ServerNetworkSummary {
    pub received_bytes_per_second: f64,
    pub transmitted_bytes_per_second: f64,
    pub packets_received: u64,
    pub packets_transmitted: u64,
    pub errors_on_received: u64,
    pub errors_on_transmitted: u64,
    pub total_received_bytes: u64,
    pub total_transmitted_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerProcessMetrics {
    pub pid: u32,
    pub name: String,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub run_time_seconds: u64,
    pub disk_read_bytes: u64,
    pub disk_written_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerHealth {
    pub status: ServerHealthStatus,
    pub issues: Vec<ServerHealthIssue>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ServerHealthIssue {
    pub kind: ServerHealthIssueKind,
    pub target: String,
    pub usage_percent: f32,
    pub threshold_percent: f32,
    pub status: ServerHealthStatus,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerHealthIssueKind {
    Cpu,
    Memory,
    Disk,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerHealthStatus {
    Green,
    Yellow,
    Red,
}
