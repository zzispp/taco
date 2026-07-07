use std::time::Instant;

use async_trait::async_trait;
use sysinfo::{Disks, MINIMUM_CPU_UPDATE_INTERVAL, Networks, ProcessesToUpdate, System};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::{
    application::{ServerMetricsCollector, SystemError, SystemResult, evaluate_dashboard_health},
    domain::{
        ServerCpuCoreMetrics, ServerCpuMetrics, ServerDashboard, ServerDiskMetrics, ServerHostInfo, ServerLoadAverage, ServerMemoryMetrics,
        ServerNetworkSummary, ServerProcessMetrics,
    },
};

use super::network_interfaces::public_ips;

const TOP_PROCESS_LIMIT: usize = 8;
const MILLIS_PER_SECOND: f64 = 1_000.0;
const PERCENT_MULTIPLIER: f64 = 100.0;

#[derive(Clone, Default)]
pub struct SysinfoServerMetricsCollector;

#[async_trait]
impl ServerMetricsCollector for SysinfoServerMetricsCollector {
    async fn collect(&self) -> SystemResult<ServerDashboard> {
        tokio::task::spawn_blocking(collect_dashboard).await.map_err(join_error)?
    }
}

fn collect_dashboard() -> SystemResult<ServerDashboard> {
    let mut system = System::new_all();
    let mut networks = Networks::new_with_refreshed_list();
    let start = Instant::now();
    std::thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);
    refresh_sources(&mut system, &mut networks);

    let disks = disks();
    let memory = memory(&system);
    let cpu = cpu(&system);
    let health = evaluate_dashboard_health(cpu.total_usage_percent, memory.usage_percent, &disks);
    let sample_duration_millis = millis(start.elapsed());

    Ok(ServerDashboard {
        host: host(&system, &disks)?,
        cpu,
        memory,
        disks,
        network: network_summary(&networks, sample_duration_millis),
        top_processes: top_processes(&system),
        health,
        sampled_at: sampled_at()?,
        sample_duration_millis,
    })
}

fn refresh_sources(system: &mut System, networks: &mut Networks) {
    system.refresh_memory();
    system.refresh_cpu_all();
    system.refresh_processes(ProcessesToUpdate::All, true);
    networks.refresh(true);
}

fn host(system: &System, disks: &[ServerDiskMetrics]) -> SystemResult<ServerHostInfo> {
    Ok(ServerHostInfo {
        hostname: System::host_name(),
        public_ips: public_ips()?,
        os_name: System::name(),
        os_version: System::long_os_version().or_else(System::os_version),
        kernel_version: System::kernel_version(),
        cpu_arch: System::cpu_arch(),
        cpu_brand: system.cpus().first().map(|cpu| cpu.brand().to_owned()),
        physical_core_count: System::physical_core_count(),
        logical_core_count: system.cpus().len(),
        uptime_seconds: System::uptime(),
        boot_time_unix_seconds: System::boot_time(),
        total_memory_bytes: system.total_memory(),
        total_disk_bytes: disks.iter().map(|disk| disk.total_bytes).sum(),
    })
}

fn cpu(system: &System) -> ServerCpuMetrics {
    let load = System::load_average();
    ServerCpuMetrics {
        total_usage_percent: system.global_cpu_usage(),
        load_average: ServerLoadAverage {
            one: load.one,
            five: load.five,
            fifteen: load.fifteen,
        },
        cores: system.cpus().iter().map(cpu_core).collect(),
    }
}

fn cpu_core(cpu: &sysinfo::Cpu) -> ServerCpuCoreMetrics {
    ServerCpuCoreMetrics {
        name: cpu.name().to_owned(),
        usage_percent: cpu.cpu_usage(),
        frequency_mhz: cpu.frequency(),
    }
}

fn memory(system: &System) -> ServerMemoryMetrics {
    let total = system.total_memory();
    let used = system.used_memory();
    let total_swap = system.total_swap();
    let used_swap = system.used_swap();
    ServerMemoryMetrics {
        total_bytes: total,
        used_bytes: used,
        free_bytes: system.free_memory(),
        available_bytes: system.available_memory(),
        usage_percent: percent(used, total),
        total_swap_bytes: total_swap,
        used_swap_bytes: used_swap,
        free_swap_bytes: system.free_swap(),
        swap_usage_percent: percent(used_swap, total_swap),
    }
}

fn disks() -> Vec<ServerDiskMetrics> {
    let mut disks = Disks::new_with_refreshed_list();
    disks.refresh(true);
    disks.iter().map(disk_metrics).collect()
}

fn disk_metrics(disk: &sysinfo::Disk) -> ServerDiskMetrics {
    let total = disk.total_space();
    let available = disk.available_space();
    let used = total.saturating_sub(available);
    ServerDiskMetrics {
        name: disk.name().to_string_lossy().into_owned(),
        mount_point: disk.mount_point().to_string_lossy().into_owned(),
        file_system: disk.file_system().to_string_lossy().into_owned(),
        total_bytes: total,
        available_bytes: available,
        used_bytes: used,
        usage_percent: percent(used, total),
    }
}

fn network_summary(networks: &Networks, sample_duration_millis: u64) -> ServerNetworkSummary {
    let seconds = sample_duration_millis as f64 / MILLIS_PER_SECOND;
    let mut summary = ServerNetworkSummary::default();

    for (_, data) in networks {
        summary.received_bytes_per_second += rate(data.received(), seconds);
        summary.transmitted_bytes_per_second += rate(data.transmitted(), seconds);
        summary.packets_received += data.packets_received();
        summary.packets_transmitted += data.packets_transmitted();
        summary.errors_on_received += data.errors_on_received();
        summary.errors_on_transmitted += data.errors_on_transmitted();
        summary.total_received_bytes += data.total_received();
        summary.total_transmitted_bytes += data.total_transmitted();
    }

    summary
}

fn top_processes(system: &System) -> Vec<ServerProcessMetrics> {
    let mut processes = system.processes().values().map(process_metrics).collect::<Vec<_>>();
    processes.sort_by(|left, right| {
        right
            .cpu_usage_percent
            .total_cmp(&left.cpu_usage_percent)
            .then_with(|| right.memory_bytes.cmp(&left.memory_bytes))
    });
    processes.truncate(TOP_PROCESS_LIMIT);
    processes
}

fn process_metrics(process: &sysinfo::Process) -> ServerProcessMetrics {
    let disk = process.disk_usage();
    ServerProcessMetrics {
        pid: process.pid().as_u32(),
        name: process.name().to_string_lossy().into_owned(),
        cpu_usage_percent: process.cpu_usage(),
        memory_bytes: process.memory(),
        virtual_memory_bytes: process.virtual_memory(),
        run_time_seconds: process.run_time(),
        disk_read_bytes: disk.read_bytes,
        disk_written_bytes: disk.written_bytes,
    }
}

fn percent(used: u64, total: u64) -> f32 {
    if total == 0 {
        return 0.0;
    }
    (used as f64 * PERCENT_MULTIPLIER / total as f64) as f32
}

fn rate(bytes: u64, seconds: f64) -> f64 {
    if seconds <= 0.0 {
        return 0.0;
    }
    bytes as f64 / seconds
}

fn millis(duration: std::time::Duration) -> u64 {
    u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
}

fn sampled_at() -> SystemResult<String> {
    OffsetDateTime::now_utc().format(&Rfc3339).map_err(time_error)
}

fn join_error(error: tokio::task::JoinError) -> SystemError {
    SystemError::Infrastructure(format!("server metrics task failed: {error}"))
}

fn time_error(error: time::error::Format) -> SystemError {
    SystemError::Infrastructure(format!("server metrics timestamp format error: {error}"))
}
