// //{\"Created\":1429987381,\"Id\":\"af78f2747e72ef24a4c4387a660ac21356d2c70301cc61d804cdabe40a0598c7\",\"Labels\":{},\"ParentId\":\"ef28fd1d12034983d1d347b5e8575d2aac12189b9ba0030912a26b1414f1e79e\",\"RepoDigests\":[\"\\u003cnone\\u003e@\\u003cnone\\u003e\"],\"RepoTags\":[\"\\u003cnone\\u003e:\\u003cnone\\u003e\"],\"Size\":0,\"VirtualSize\":5029718}
#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Image {
  pub Created: u64,
  pub Id: String,
  pub ParentId: String,
  //pub Labels: ???,
  pub RepoTags: Vec<String>,
  pub RepoDigests: Vec<String>,
  pub Size: u64,
  pub VirtualSize: u64
}

// {\"Command\":\"/opt/zookeeper-3.4.5/bin/zkServer.sh start-foreground\",\"Created\":1430194916,\"Id\":\"160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36\",\"Image\":\"jplock/zookeeper:latest\",\"Labels\":{},\"Names\":[\"/stoic_perlman\"],\"Ports\":[{\"PrivatePort\":2181,\"Type\":\"tcp\"},{\"PrivatePort\":2888,\"Type\":\"tcp\"},{\"PrivatePort\":3888,\"Type\":\"tcp\"}],\"Status\":\"Up About a minute\"}
#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Container {
  pub Created: u64,
  pub Command: String,
  pub Id: String,
  pub Image: String,
  //pub Labels: ???,
  pub Names: Vec<String>,
  pub Ports: Vec<Port>,
  pub Status: String,
}


#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Port {
  pub IP: Option<String>,
  pub PrivatePort: u64,
  pub PublicPort: Option<u64>,
  pub Type: String
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Stats {
  pub read: String,
  pub network: Network,
  pub memory_stats: MemoryStats,
  pub cpu_stats: CpuStats
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Network {
  pub rx_dropped: u64,
  pub rx_bytes: u64,
  pub rx_errors: u64,
  pub tx_packets: u64,
  pub tx_dropped: u64,
  pub rx_packets: u64,
  pub tx_errors: u64,
  pub tx_bytes: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct MemoryStats {
  pub max_usage: u64,
  pub usage: u64,
  pub failcnt: u64,
  pub limit: u64,
  pub stats: MemoryStat
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
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
  pub swap: u64,
  pub total_swap: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct CpuStats {
  pub cpu_usage: CpuUsage,
  pub system_cpu_usage: u64,
  pub throttling_data: ThrottlingData
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct CpuUsage {
  pub percpu_usage: Vec<u64>,
  pub usage_in_usermode: u64,
  pub total_usage: u64,
  pub usage_in_kernelmode: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct ThrottlingData {
  pub periods: u64,
  pub throttled_periods: u64,
  pub throttled_time: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct BlkioStats {
  pub io_service_bytes_recursive: Vec<BlkioStat>,
  pub io_serviced_recursive: Vec<BlkioStat>,
  pub io_queue_recursive: Vec<BlkioStat>,
  pub io_service_time_recursive: Vec<BlkioStat>,
  pub io_wait_time_recursive: Vec<BlkioStat>,
  pub io_merged_recursive: Vec<BlkioStat>,
  pub io_time_recursive: Vec<BlkioStat>,
  pub sectors_recursive: Vec<BlkioStat>
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct BlkioStat {
  pub major: u64,
  pub minor: u64,
  pub op: String,
  pub value: u64
}
