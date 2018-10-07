//! Rust representations of docker json structures

use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub description: String,
    pub is_official: bool,
    pub is_automated: bool,
    pub name: String,
    pub star_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Image {
    pub created: u64,
    pub id: String,
    pub parent_id: String,
    pub labels: Option<HashMap<String, String>>,
    pub repo_tags: Option<Vec<String>>,
    pub repo_digests: Option<Vec<String>>,
    pub virtual_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ImageDetails {
    pub architecture: String,
    pub author: String,
    pub comment: String,
    pub config: Config,
    pub created: String,
    pub docker_version: String,
    pub id: String,
    pub os: String,
    pub parent: String,
    pub size: u64,
    pub virtual_size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Container {
    pub created: u64,
    pub command: String,
    pub id: String,
    pub image: String,
    pub labels: HashMap<String, String>,
    pub names: Vec<String>,
    pub ports: Vec<Port>,
    pub status: String,
    pub size_rw: Option<u64>,
    pub size_root_fs: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerDetails {
    pub app_armor_profile: String,
    pub args: Vec<String>,
    pub config: Config,
    pub created: String,
    pub driver: String,
    // pub ExecIDs: ??
    pub host_config: HostConfig,
    pub hostname_path: String,
    pub hosts_path: String,
    pub log_path: String,
    pub id: String,
    pub image: String,
    pub mount_label: String,
    pub name: String,
    pub network_settings: NetworkSettings,
    pub path: String,
    pub process_label: String,
    pub resolv_conf_path: String,
    pub restart_count: u64,
    pub state: State,
    pub mounts: Vec<Mount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Mount {
    pub source: String,
    pub destination: String,
    pub mode: String,
    #[serde(rename = "RW")]
    pub rw: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct State {
    pub error: String,
    pub exit_code: u64,
    pub finished_at: String,
    pub oom_killed: bool,
    pub paused: bool,
    pub pid: u64,
    pub restarting: bool,
    pub running: bool,
    pub started_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkSettings {
    pub bridge: String,
    pub gateway: String,
    #[serde(rename = "IPAddress")]
    pub ip_address: String,
    #[serde(rename = "IPPrefixLen")]
    pub ip_prefix_len: u64,
    pub mac_address: String, /*    pub PortMapping: Option<???>,
                             *   pub Ports: Option<???> */
    pub networks: HashMap<String, NetworkEntry>
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkEntry {
    #[serde(rename = "NetworkID")]
    pub network_id: String,
    #[serde(rename = "EndpointID")]
    pub endpoint_id: String,
    pub gateway: String,
    #[serde(rename = "IPAddress")]
    pub ip_address: String,
    #[serde(rename = "IPPrefixLen")]
    pub ip_prefix_len: u64,
    #[serde(rename = "IPv6Gateway")]
    pub ipv6_gateway: String,
    #[serde(rename = "GlobalIPv6Address")]
    pub global_ipv6_address: String,
    #[serde(rename = "GlobalIPv6PrefixLen")]
    pub global_ipv6_prefix_len: u64,
    pub mac_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct HostConfig {
    pub cgroup_parent: Option<String>,
    #[serde(rename = "ContainerIDFile")]
    pub container_id_file: String,
    pub cpu_shares: Option<u64>,
    pub cpuset_cpus: Option<String>,
    pub memory: Option<u64>,
    pub memory_swap: Option<i64>,
    pub network_mode: String,
    pub pid_mode: Option<String>,
    // pub PortBindings: ???
    pub privileged: bool,
    pub publish_all_ports: bool,
    pub readonly_rootfs: Option<bool>, /* pub RestartPolicy: ???
                                       * pub SecurityOpt: Option<???>,
                                       * pub Ulimits: Option<???>
                                       * pub VolumesFrom: Option<??/> */
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    pub attach_stderr: bool,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub cmd: Option<Vec<String>>,
    pub domainname: String,
    pub entrypoint: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    // ExposedPorts
    pub hostname: String,
    pub image: String,
    pub labels: Option<HashMap<String, String>>,
    // pub MacAddress: String,
    pub on_build: Option<Vec<String>>,
    // pub NetworkDisabled: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub tty: bool,
    pub user: String,
    pub working_dir: String,
}

impl Config {
    pub fn env(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self.env {
            Some(ref vars) => {
                for e in vars {
                    let pair: Vec<&str> = e.split("=").collect();
                    map.insert(pair[0].to_owned(), pair[1].to_owned());
                }
            }
            _ => (),
        };
        map
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Port {
    pub ip: Option<String>,
    pub private_port: u64,
    pub public_port: Option<u64>,
    #[serde(rename = "Type")]
    pub typ: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Stats {
    pub read: String,
    pub networks: HashMap<String, Network>,
    pub memory_stats: MemoryStats,
    pub blkio_stats: BlkioStats,
    pub cpu_stats: CpuStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Network {
    pub rx_dropped: u64,
    pub rx_bytes: u64,
    pub rx_errors: u64,
    pub tx_packets: u64,
    pub tx_dropped: u64,
    pub rx_packets: u64,
    pub tx_errors: u64,
    pub tx_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct IPAM {
    pub driver: String,
    pub config: Vec<HashMap<String, String>>,
    pub options: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkDetails {
    pub name: String,
    pub id: String,
    pub scope: String,
    pub driver: String,
    pub enable_i_pv6: bool,
    pub ipam: IPAM,
    pub internal: bool,
    pub attachable: bool,
    pub containers: HashMap<String, NetworkContainerDetails>,
    pub options: Option<HashMap<String, String>>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkContainerDetails {
    pub endpoint_id: String,
    pub mac_address: String,
    pub i_pv4_address: String,
    pub i_pv6_address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NetworkCreateInfo {
    pub id: String,
    pub warning: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryStats {
    pub max_usage: u64,
    pub usage: u64,
    pub failcnt: Option<u64>,
    pub limit: u64,
    pub stats: MemoryStat,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryStat {
    pub total_pgmajfault: u64,
    pub cache: u64,
    pub mapped_file: u64,
    pub total_inactive_file: u64,
    pub pgpgout: u64,
    pub rss: u64,
    pub total_mapped_file: u64,
    pub writeback: u64,
    pub unevictable: u64,
    pub pgpgin: u64,
    pub total_unevictable: u64,
    pub pgmajfault: u64,
    pub total_rss: u64,
    pub total_rss_huge: u64,
    pub total_writeback: u64,
    pub total_inactive_anon: u64,
    pub rss_huge: u64,
    pub hierarchical_memory_limit: u64,
    pub hierarchical_memsw_limit: u64,
    pub total_pgfault: u64,
    pub total_active_file: u64,
    pub active_anon: u64,
    pub total_active_anon: u64,
    pub total_pgpgout: u64,
    pub total_cache: u64,
    pub inactive_anon: u64,
    pub active_file: u64,
    pub pgfault: u64,
    pub inactive_file: u64,
    pub total_pgpgin: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpuStats {
    pub cpu_usage: CpuUsage,
    pub system_cpu_usage: u64,
    pub throttling_data: ThrottlingData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CpuUsage {
    pub percpu_usage: Vec<u64>,
    pub usage_in_usermode: u64,
    pub total_usage: u64,
    pub usage_in_kernelmode: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ThrottlingData {
    pub periods: u64,
    pub throttled_periods: u64,
    pub throttled_time: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlkioStats {
    pub io_service_bytes_recursive: Vec<BlkioStat>,
    pub io_serviced_recursive: Vec<BlkioStat>,
    pub io_queue_recursive: Vec<BlkioStat>,
    pub io_service_time_recursive: Vec<BlkioStat>,
    pub io_wait_time_recursive: Vec<BlkioStat>,
    pub io_merged_recursive: Vec<BlkioStat>,
    pub io_time_recursive: Vec<BlkioStat>,
    pub sectors_recursive: Vec<BlkioStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlkioStat {
    pub major: u64,
    pub minor: u64,
    pub op: String,
    pub value: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Change {
    pub kind: u64,
    pub path: String,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Top {
    pub titles: Vec<String>,
    pub processes: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Version {
    pub api_version: String,
    pub version: String,
    pub git_commit: String,
    pub go_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Info {
    pub containers: u64,
    pub images: u64,
    pub driver: String,
    pub docker_root_dir: String,
    pub driver_status: Vec<Vec<String>>,
    pub id: String,
    pub kernel_version: String,
    // pub Labels: Option<???>,
    pub mem_total: u64,
    pub memory_limit: bool,
    pub ncpu: u64,
    pub n_events_listener: u64,
    pub n_goroutines: u64,
    pub name: String,
    pub operating_system: String,
    // pub RegistryConfig:???
    pub swap_limit: bool,
    pub system_time: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerCreateInfo {
    pub id: String,
    pub warnings: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct History {
    pub id: String,
    pub created: u64,
    pub created_by: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Exit {
    pub status_code: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Event {
    #[serde(rename = "Type")]
    pub typ: String,
    pub action: String,
    pub actor: Actor,
    pub status: Option<String>,
    pub id: Option<String>,
    pub from: Option<String>,
    pub time: u64,
    pub time_nano: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Actor {
    pub id: String,
    pub attributes: HashMap<String, String>,
}

#[derive(Clone, Debug)]
pub enum Status {
    Untagged(String),
    Deleted(String),
}
