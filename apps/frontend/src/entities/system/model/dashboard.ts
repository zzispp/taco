export type ServerDashboard = {
  host: ServerHostInfo;
  cpu: ServerCpuMetrics;
  memory: ServerMemoryMetrics;
  disks: ServerDiskMetrics[];
  network: ServerNetworkSummary;
  top_processes: ServerProcessMetrics[];
  health: ServerHealth;
  sampled_at: string;
  sample_duration_millis: number;
};

export type ServerHostInfo = {
  hostname?: string | null;
  public_ips: string[];
  os_name?: string | null;
  os_version?: string | null;
  kernel_version?: string | null;
  cpu_arch: string;
  cpu_brand?: string | null;
  physical_core_count?: number | null;
  logical_core_count: number;
  uptime_seconds: number;
  boot_time_unix_seconds: number;
  total_memory_bytes: number;
  total_disk_bytes: number;
};

export type ServerCpuMetrics = {
  total_usage_percent: number;
  load_average: ServerLoadAverage;
  cores: ServerCpuCoreMetrics[];
};

export type ServerLoadAverage = {
  one: number;
  five: number;
  fifteen: number;
};

export type ServerCpuCoreMetrics = {
  name: string;
  usage_percent: number;
  frequency_mhz: number;
};

export type ServerMemoryMetrics = {
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  available_bytes: number;
  usage_percent: number;
  total_swap_bytes: number;
  used_swap_bytes: number;
  free_swap_bytes: number;
  swap_usage_percent: number;
};

export type ServerDiskMetrics = {
  name: string;
  mount_point: string;
  file_system: string;
  total_bytes: number;
  available_bytes: number;
  used_bytes: number;
  usage_percent: number;
};

export type ServerNetworkSummary = {
  received_bytes_per_second: number;
  transmitted_bytes_per_second: number;
  packets_received: number;
  packets_transmitted: number;
  errors_on_received: number;
  errors_on_transmitted: number;
  total_received_bytes: number;
  total_transmitted_bytes: number;
};

export type ServerProcessMetrics = {
  pid: number;
  name: string;
  cpu_usage_percent: number;
  memory_bytes: number;
  virtual_memory_bytes: number;
  run_time_seconds: number;
  disk_read_bytes: number;
  disk_written_bytes: number;
};

export type ServerHealth = {
  status: ServerHealthStatus;
  issues: ServerHealthIssue[];
};

export type ServerHealthStatus = 'green' | 'yellow' | 'red';

export type ServerHealthIssue = {
  kind: 'cpu' | 'memory' | 'disk';
  target: string;
  usage_percent: number;
  threshold_percent: number;
  status: ServerHealthStatus;
};
