//! Rust representations of docker json structures

use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub description: String,
    pub is_official: bool,
    pub is_trusted: bool,
    pub name: String,
    pub star_count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Image {
    pub Created: u64,
    pub Id: String,
    pub ParentId: String,
    pub Labels: Option<HashMap<String, String>>,
    pub RepoTags: Vec<String>,
    pub RepoDigests: Option<Vec<String>>,
    pub VirtualSize: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ImageDetails {
    pub Architecture: String,
    pub Author: String,
    pub Comment: String,
    pub Config: Config,
    pub Created: String,
    pub DockerVersion: String,
    pub Id: String,
    pub Os: String,
    pub Parent: String,
    pub Size: u64,
    pub VirtualSize: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Container {
    pub Created: u64,
    pub Command: String,
    pub Id: String,
    pub Image: String,
    pub Labels: HashMap<String, String>,
    pub Names: Vec<String>,
    pub Ports: Vec<Port>,
    pub Status: String,
    pub SizeRw: Option<u64>,
    pub SizeRootFs: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ContainerDetails {
    pub AppArmorProfile: String,
    pub Args: Vec<String>,
    pub Config: Config,
    pub Created: String,
    pub Driver: String,
    // pub ExecIDs: ??
    pub HostConfig: HostConfig,
    pub HostnamePath: String,
    pub HostsPath: String,
    pub LogPath: String,
    pub Id: String,
    pub Image: String,
    pub MountLabel: String,
    pub NetworkSettings: NetworkSettings,
    pub Path: String,
    pub ProcessLabel: String,
    pub ResolvConfPath: String,
    pub RestartCount: u64,
    pub State: State,
    pub Mounts: Vec<Mount>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Mount {
    pub Source: String,
    pub Destination: String,
    pub Mode: String,
    pub RW: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct State {
    pub Error: String,
    pub ExitCode: u64,
    pub FinishedAt: String,
    pub OOMKilled: bool,
    pub Paused: bool,
    pub Pid: u64,
    pub Restarting: bool,
    pub Running: bool,
    pub StartedAt: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NetworkSettings {
    pub Bridge: String,
    pub Gateway: String,
    pub IPAddress: String,
    pub IPPrefixLen: u64,
    pub MacAddress: String, /*    pub PortMapping: Option<???>,
                             *   pub Ports: Option<???> */
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct HostConfig {
    pub CgroupParent: Option<String>,
    pub ContainerIDFile: String,
    pub CpuShares: Option<u64>,
    pub CpusetCpus: Option<String>,
    pub Memory: Option<u64>,
    pub MemorySwap: Option<u64>,
    pub NetworkMode: String,
    pub PidMode: Option<String>,
    // pub PortBindings: ???
    pub Privileged: bool,
    pub PublishAllPorts: bool,
    pub ReadonlyRootfs: Option<bool>, /* pub RestartPolicy: ???
                                       * pub SecurityOpt: Option<???>,
                                       * pub Ulimits: Option<???>
                                       * pub VolumesFrom: Option<??/> */
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Config {
    pub AttachStderr: bool,
    pub AttachStdin: bool,
    pub AttachStdout: bool,
    pub Cmd: Option<Vec<String>>,
    pub Domainname: String,
    pub Entrypoint: Option<Vec<String>>,
    pub Env: Option<Vec<String>>,
    // ExposedPorts
    pub Hostname: String,
    pub Image: String,
    pub Labels: HashMap<String, String>,
    // pub MacAddress: String,
    pub OnBuild: Option<Vec<String>>,
    // pub NetworkDisabled: bool,
    pub OpenStdin: bool,
    pub StdinOnce: bool,
    pub Tty: bool,
    pub User: String,
    pub WorkingDir: String,
}

impl Config {
    pub fn env(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        match self.Env {
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
#[allow(non_snake_case)]
pub struct Port {
    pub IP: Option<String>,
    pub PrivatePort: u64,
    pub PublicPort: Option<u64>,
    pub Type: String,
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
#[allow(non_snake_case)]
pub struct IPAM {
    pub Driver: String,
    pub Config: Vec<HashMap<String, String>>,
    pub Options: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NetworkDetails {
    pub Name: String,
    pub Id: String,
    pub Scope: String,
    pub Driver: String,
    pub EnableIPv6: bool,
    pub IPAM: IPAM,
    pub Internal: bool,
    pub Attachable: bool,
    pub Containers: HashMap<String, NetworkContainerDetails>,
    pub Options: Option<HashMap<String, String>>,
    pub Labels: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NetworkContainerDetails {
    pub EndpointID: String,
    pub MacAddress: String,
    pub IPv4Address: String,
    pub IPv6Address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct NetworkCreateInfo {
    pub Id: String,
    pub Warning: String,
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
    pub swap: Option<u64>,
    pub total_swap: Option<u64>,
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
#[allow(non_snake_case)]
pub struct Change {
    pub Kind: u64,
    pub Path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Top {
    pub Titles: Vec<String>,
    pub Processes: Vec<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Version {
    pub ApiVersion: String,
    pub Version: String,
    pub GitCommit: String,
    pub GoVersion: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Info {
    pub Containers: u64,
    pub Images: u64,
    pub Driver: String,
    pub DockerRootDir: String,
    pub DriverStatus: Vec<Vec<String>>,
    pub ID: String,
    pub KernelVersion: String,
    // pub Labels: Option<???>,
    pub MemTotal: u64,
    pub MemoryLimit: bool,
    pub NCPU: u64,
    pub NEventsListener: u64,
    pub NGoroutines: u64,
    pub Name: String,
    pub OperatingSystem: String,
    // pub RegistryConfig:???
    pub SwapLimit: bool,
    pub SystemTime: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ContainerCreateInfo {
    pub Id: String,
    pub Warnings: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct History {
    pub Id: String,
    pub Created: u64,
    pub CreatedBy: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Exit {
    pub StatusCode: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct Event {
    pub status: Option<String>,
    pub id: Option<String>,
    pub from: Option<String>,
    pub time: u64,
    pub timeNano: u64,
}

#[derive(Clone, Debug)]
pub enum Status {
    Untagged(String),
    Deleted(String),
}
