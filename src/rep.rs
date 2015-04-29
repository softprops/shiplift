// {\"description\":\"Lightweight nightly Rust build including Cargo and GDB\",\"is_official\":false,\"is_trusted\":true,\"name\":\"schickling/rust\",\"star_count\":7}
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct SearchResult {
  pub description: String,
  pub is_official: bool,
  pub is_trusted: bool,
  pub name: String,
  pub star_count: u64
}

//{\"Created\":1429987381,\"Id\":\"af78f2747e72ef24a4c4387a660ac21356d2c70301cc61d804cdabe40a0598c7\",\"Labels\":{},\"ParentId\":\"ef28fd1d12034983d1d347b5e8575d2aac12189b9ba0030912a26b1414f1e79e\",\"RepoDigests\":[\"\\u003cnone\\u003e@\\u003cnone\\u003e\"],\"RepoTags\":[\"\\u003cnone\\u003e:\\u003cnone\\u003e\"],\"Size\":0,\"VirtualSize\":5029718}
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

// "{\"Architecture\":\"amd64\",\"Author\":\"\",\"Comment\":\"\",\"Config\":{\"AttachStderr\":false,\"AttachStdin\":false,\"AttachStdout\":false,\"Cmd\":[\"redis-server\"],\"CpuShares\":0,\"Cpuset\":\"\",\"Domainname\":\"\",\"Entrypoint\":[\"/entrypoint.sh\"],\"Env\":[\"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\",\"REDIS_VERSION=3.0.0\",\"REDIS_DOWNLOAD_URL=http://download.redis.io/releases/redis-3.0.0.tar.gz\",\"REDIS_DOWNLOAD_SHA1=c75fd32900187a7c9f9d07c412ea3b3315691c65\"],\"ExposedPorts\":{\"6379/tcp\":{}},\"Hostname\":\"134fc64619a1\",\"Image\":\"54ca92b7c8d7dc34fd868a9386b2fcd220c4cbf4aa40211eb999004510ad64a8\",\"Labels\":{},\"MacAddress\":\"\",\"Memory\":0,\"MemorySwap\":0,\"NetworkDisabled\":false,\"OnBuild\":[],\"OpenStdin\":false,\"PortSpecs\":null,\"StdinOnce\":false,\"Tty\":false,\"User\":\"\",\"Volumes\":{\"/data\":{}},\"WorkingDir\":\"/data\"},\"Container\":\"c3eac0656f4346c207388cf8cb488aef3451709680379adc700ab7d515900274\",\"ContainerConfig\":{\"AttachStderr\":false,\"AttachStdin\":false,\"AttachStdout\":false,\"Cmd\":[\"/bin/sh\",\"-c\",\"#(nop) CMD [\\\"redis-server\\\"]\"],\"CpuShares\":0,\"Cpuset\":\"\",\"Domainname\":\"\",\"Entrypoint\":[\"/entrypoint.sh\"],\"Env\":[\"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\",\"REDIS_VERSION=3.0.0\",\"REDIS_DOWNLOAD_URL=http://download.redis.io/releases/redis-3.0.0.tar.gz\",\"REDIS_DOWNLOAD_SHA1=c75fd32900187a7c9f9d07c412ea3b3315691c65\"],\"ExposedPorts\":{\"6379/tcp\":{}},\"Hostname\":\"134fc64619a1\",\"Image\":\"54ca92b7c8d7dc34fd868a9386b2fcd220c4cbf4aa40211eb999004510ad64a8\",\"Labels\":{},\"MacAddress\":\"\",\"Memory\":0,\"MemorySwap\":0,\"NetworkDisabled\":false,\"OnBuild\":[],\"OpenStdin\":false,\"PortSpecs\":null,\"StdinOnce\":false,\"Tty\":false,\"User\":\"\",\"Volumes\":{\"/data\":{}},\"WorkingDir\":\"/data\"},\"Created\":\"2015-04-22T06:53:04.903659607Z\",\"DockerVersion\":\"1.6.0\",\"Id\":\"06a1f75304ba07498873a8c2a0861c19c8994cf676714a891791fab8760fff6a\",\"Os\":\"linux\",\"Parent\":\"54ca92b7c8d7dc34fd868a9386b2fcd220c4cbf4aa40211eb999004510ad64a8\",\"Size\":0,\"VirtualSize\":111053086}\n"
#[derive(Debug, RustcEncodable, RustcDecodable)]
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

// {"AppArmorProfile":"","Args":["start-foreground"],"Config":{"AttachStderr":true,"AttachStdin":false,"AttachStdout":true,"Cmd":["start-foreground"],"CpuShares":0,"Cpuset":"","Domainname":"","Entrypoint":["/opt/zookeeper-3.4.5/bin/zkServer.sh"],"Env":["HOME=/","PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin","JAVA_HOME=/usr/lib/jvm/java-7-openjdk-amd64"],"ExposedPorts":{"2181/tcp":{},"2888/tcp":{},"3888/tcp":{}},"Hostname":"160bbff9ff12","Image":"jplock/zookeeper","Labels":{},"MacAddress":"","Memory":0,"MemorySwap":0,"NetworkDisabled":false,"OnBuild":null,"OpenStdin":false,"PortSpecs":null,"StdinOnce":false,"Tty":false,"User":"","Volumes":null,"WorkingDir":""},"Created":"2015-04-28T04:21:56.570106707Z","Driver":"aufs","ExecDriver":"native-0.2","ExecIDs":null,"HostConfig":{"Binds":null,"CapAdd":null,"CapDrop":null,"CgroupParent":"","ContainerIDFile":"","CpuShares":0,"CpusetCpus":"","Devices":[],"Dns":null,"DnsSearch":null,"ExtraHosts":null,"IpcMode":"","Links":null,"LogConfig":{"Config":null,"Type":"json-file"},"LxcConf":[],"Memory":0,"MemorySwap":0,"NetworkMode":"bridge","PidMode":"","PortBindings":{},"Privileged":false,"PublishAllPorts":false,"ReadonlyRootfs":false,"RestartPolicy":{"MaximumRetryCount":0,"Name":"no"},"SecurityOpt":null,"Ulimits":null,"VolumesFrom":null},"HostnamePath":"/mnt/sda1/var/lib/docker/containers/160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36/hostname","HostsPath":"/mnt/sda1/var/lib/docker/containers/160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36/hosts","Id":"160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36","Image":"9ce81845fa8fd9ebbed4b607fa68f1b7bdb2dd9365e6b330fa08069e367a2d6e","LogPath":"/mnt/sda1/var/lib/docker/containers/160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36/160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36-json.log","MountLabel":"","Name":"/stoic_perlman","NetworkSettings":{"Bridge":"docker0","Gateway":"172.17.42.1","GlobalIPv6Address":"","GlobalIPv6PrefixLen":0,"IPAddress":"172.17.0.10","IPPrefixLen":16,"IPv6Gateway":"","LinkLocalIPv6Address":"fe80::42:acff:fe11:a","LinkLocalIPv6PrefixLen":64,"MacAddress":"02:42:ac:11:00:0a","PortMapping":null,"Ports":{"2181/tcp":null,"2888/tcp":null,"3888/tcp":null}},"Path":"/opt/zookeeper-3.4.5/bin/zkServer.sh","ProcessLabel":"","ResolvConfPath":"/mnt/sda1/var/lib/docker/containers/160bbff9ff12e10f73c16a4f20d5ac785bf43066017e28cb24d53cc1c128ee36/resolv.conf","RestartCount":0,"State":{"Dead":false,"Error":"","ExitCode":0,"FinishedAt":"0001-01-01T00:00:00Z","OOMKilled":false,"Paused":false,"Pid":6635,"Restarting":false,"Running":true,"StartedAt":"2015-04-28T04:21:57.580505523Z"},"Volumes":{},"VolumesRW":{}}
#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct ContainerDetails {
  pub AppArmorProfile: String,
  pub Args: Vec<String>,
  pub Config: Config,
  pub Created: String,
  pub Driver: String,
  pub ExecDriver: String,
  //pub ExecIDs: ??
  pub HostConfig: HostConfig
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct HostConfig {
  pub CgroupParent: String,
  pub ContainerIDFile: String,
  pub CpuShares: u64,
  pub CpusetCpus: String,
  pub Memory: u64,
  pub MemorySwap: u64,
  pub NetworkMode: String,
  pub PidMode: String,
  //pub PortBindings: ???
  pub Privileged: bool,
  pub PublishAllPorts: bool,
  pub ReadonlyRootfs: bool
  //pub RestartPolicy: ???
  //pub SecurityOpt: Option<???>,
  //pub Ulimits: Option<???>
 // pub VolumesFrom: Option<??/>
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Config {
  AttachStderr: bool,
  AttachStdin: bool,
  AttachStdout: bool,
  Cmd: Vec<String>,
  CpuShares: u64,
  Cpuset: String,
  Domainname: String,
  Entrypoint: Vec<String>,
  Env: Vec<String>,
  //ExposedPorts
  Hostname: String,
  Image: String,
  //Labels:???
  MacAddress: String,
  Memory: u64,
  MemorySwap: u64,
  NetworkDisabled: bool,
  //OnBuild: Option<String>,
  OpenStdin: bool,
  //PortSpecs: ???,
  StdinOnce: bool,
  Tty: bool,
  User: String,
  //Volumes: ??,
  WorkingDir: String
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
pub struct Stats {
  pub read: String,
  pub network: Network,
  pub memory_stats: MemoryStats,
  pub cpu_stats: CpuStats
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
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
pub struct MemoryStats {
  pub max_usage: u64,
  pub usage: u64,
  pub failcnt: u64,
  pub limit: u64,
  pub stats: MemoryStat
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
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
pub struct CpuStats {
  pub cpu_usage: CpuUsage,
  pub system_cpu_usage: u64,
  pub throttling_data: ThrottlingData
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct CpuUsage {
  pub percpu_usage: Vec<u64>,
  pub usage_in_usermode: u64,
  pub total_usage: u64,
  pub usage_in_kernelmode: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub struct ThrottlingData {
  pub periods: u64,
  pub throttled_periods: u64,
  pub throttled_time: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
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
pub struct BlkioStat {
  pub major: u64,
  pub minor: u64,
  pub op: String,
  pub value: u64
}

#[derive(Debug, RustcEncodable, RustcDecodable)]
#[allow(non_snake_case)]
pub struct Change {
  pub Kind: u64,
  pub Path: String
}
