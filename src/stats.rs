use log::error;
use log::info;
use std::{io::Error, thread};

use serde::Serialize;
use systemstat::{
    saturating_sub_bytes, ByteSize, Duration, IpAddr, NetworkAddrs, Platform, System,
};

const BYTES_PER_MB: u64 = 1_000_000;

/// All system stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllStats {
    /// General system stats
    pub general: GeneralStats,
    /// CPU stats
    pub cpu: CpuStats,
    /// Memory stats
    pub memory: Option<MemoryStats>,
    /// Stats for each mounted filesystem
    pub filesystems: Option<Vec<MountStats>>,
    /// Network stats
    pub network: NetworkStats,
}

impl AllStats {
    /// Updates stats using the provided system.
    /// # Params
    /// * `sys` - The system to get stats from.
    /// * `cpu_sample_duration` - The amount of time to take to sample CPU load. Note that this function will block the thread it's in for this duration before returning.
    pub fn update(&mut self, sys: &System, cpu_sample_duration: Duration) {
        self.general.update(sys);
        self.cpu.update(sys, cpu_sample_duration);
        self.memory = MemoryStats::from(sys);
        self.filesystems = MountStats::from(sys);
        self.network.update(sys);
    }
}

impl Default for AllStats {
    fn default() -> Self {
        AllStats {
            general: GeneralStats::default(),
            cpu: CpuStats::default(),
            memory: None,
            filesystems: None,
            network: NetworkStats::default(),
        }
    }
}

/// General system stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralStats {
    /// Number of seconds the system has been running
    uptime_seconds: Option<u64>,
    /// Boot time in seconds since the UNIX epoch
    boot_timestamp: Option<i64>,
    /// Load average values for the system
    load_averages: Option<LoadAverages>,
}

impl Default for GeneralStats {
    fn default() -> Self {
        GeneralStats {
            uptime_seconds: None,
            boot_timestamp: None,
            load_averages: None,
        }
    }
}

/// Load average values
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadAverages {
    /// Load average over the last minute
    one_minute: f32,
    /// Load average over the last 5 minutes
    five_minutes: f32,
    /// Load average over the last 15 minutes
    fifteen_minutes: f32,
}

impl GeneralStats {
    /// Gets general stats for the provided system.
    pub fn from(sys: &System) -> GeneralStats {
        GeneralStats {
            uptime_seconds: Self::get_uptime_seconds(sys),
            boot_timestamp: Self::get_boot_timestamp(sys),
            load_averages: Self::get_load_averages(sys),
        }
    }

    /// Updates stats using the provided system.
    pub fn update(&mut self, sys: &System) {
        self.uptime_seconds = Self::get_uptime_seconds(sys);
        self.boot_timestamp = Self::get_boot_timestamp(sys);
        self.load_averages = Self::get_load_averages(sys);
    }

    fn get_uptime_seconds(sys: &System) -> Option<u64> {
        match sys.uptime() {
            Ok(x) => Some(x.as_secs()),
            Err(e) => {
                log("Error getting uptime: ", e);
                None
            }
        }
    }

    fn get_boot_timestamp(sys: &System) -> Option<i64> {
        match sys.boot_time() {
            Ok(boot_time) => Some(boot_time.timestamp()),
            Err(e) => {
                log("Error getting boot time: ", e);
                None
            }
        }
    }

    fn get_load_averages(sys: &System) -> Option<LoadAverages> {
        match sys.load_average() {
            Ok(x) => Some(LoadAverages {
                one_minute: x.one,
                five_minutes: x.five,
                fifteen_minutes: x.fifteen,
            }),
            Err(e) => {
                log("Error getting load average: ", e);
                None
            }
        }
    }
}

/// CPU stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuStats {
    /// Load percentages for each logical CPU
    per_logical_cpu_load_percent: Option<Vec<f32>>,
    /// Load percentage of the CPU as a whole
    aggregate_load_percent: Option<f32>,
    /// Temperature of the CPU in degrees Celsius
    temp_celsius: Option<f32>,
}

impl Default for CpuStats {
    fn default() -> Self {
        CpuStats {
            per_logical_cpu_load_percent: None,
            aggregate_load_percent: None,
            temp_celsius: None,
        }
    }
}

impl CpuStats {
    /// Gets CPU stats for the provided system.
    /// # Params
    /// * `sys` - The system to get stats from.
    /// * `sample_duration` - The amount of time to take to sample CPU load. Note that this function will block the thread it's in for this duration before returning.
    pub fn from(sys: &System, sample_duration: Duration) -> CpuStats {
        let (per_logical_cpu_load_percent, aggregate_load_percent) =
            Self::get_load(sys, sample_duration);

        CpuStats {
            per_logical_cpu_load_percent,
            aggregate_load_percent,
            temp_celsius: Self::get_temp_celsius(sys),
        }
    }

    /// Updates stats using the provided system.
    /// # Params
    /// * `sys` - The system to get stats from.
    /// * `sample_duration` - The amount of time to take to sample CPU load. Note that this function will block the thread it's in for this duration before returning.
    pub fn update(&mut self, sys: &System, sample_duration: Duration) {
        let (per_logical_cpu_load_percent, aggregate_load_percent) =
            Self::get_load(sys, sample_duration);

        self.per_logical_cpu_load_percent = per_logical_cpu_load_percent;
        self.aggregate_load_percent = aggregate_load_percent;
        self.temp_celsius = Self::get_temp_celsius(sys);
    }

    fn get_load(sys: &System, sample_duration: Duration) -> (Option<Vec<f32>>, Option<f32>) {
        let cpu_load = sys.cpu_load();
        let cpu_load_aggregate = sys.cpu_load_aggregate();
        thread::sleep(sample_duration);
        let per_logical_cpu_load_percent = match cpu_load {
            Ok(x) => match x.done() {
                Ok(cpus) => Some(cpus.iter().map(|cpu| (1.0 - cpu.idle) * 100.0).collect()),
                Err(e) => {
                    log("Error getting per logical CPU load: ", e);
                    None
                }
            },
            Err(e) => {
                log("Error getting per logical CPU load: ", e);
                None
            }
        };

        let aggregate_load_percent = match cpu_load_aggregate {
            Ok(x) => match x.done() {
                Ok(cpu) => Some((1.0 - cpu.idle) * 100.0),
                Err(e) => {
                    log("Error getting aggregate CPU load: ", e);
                    None
                }
            },
            Err(e) => {
                log("Error getting aggregate CPU load: ", e);
                None
            }
        };

        (per_logical_cpu_load_percent, aggregate_load_percent)
    }

    fn get_temp_celsius(sys: &System) -> Option<f32> {
        match sys.cpu_temp() {
            Ok(x) => Some(x),
            Err(e) => {
                log("Error getting CPU temperature: ", e);
                None
            }
        }
    }
}

/// Memory stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryStats {
    /// Megabytes of memory used
    used_mb: u64,
    /// Megabytes of memory total
    total_mb: u64,
}

impl MemoryStats {
    /// Gets memory stats for the provided system. Returns `None` if an error occurs.
    pub fn from(sys: &System) -> Option<MemoryStats> {
        match sys.memory() {
            Ok(mem) => {
                let used_mem = saturating_sub_bytes(mem.total, mem.free);
                Some(MemoryStats {
                    used_mb: bytes_to_mb(used_mem),
                    total_mb: bytes_to_mb(mem.total),
                })
            }
            Err(e) => {
                log("Error getting memory usage: ", e);
                None
            }
        }
    }
}

/// Stats for a mounted filesystem
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MountStats {
    /// Type of filesystem (NTFS, ext3, etc.)
    fs_type: String,
    /// Name of the device corresponding to this mount
    mounted_from: String,
    /// Root path corresponding to this mount
    mounted_on: String,
    /// Space of this mount used in megabytes
    used_mb: u64,
    /// Total space for this mount in megabytes
    total_mb: u64,
}

impl MountStats {
    /// Gets a list of mount stats for the provided system. Only mounts with more than 0 bytes of total space are included. Returns `None` if an error occurs.
    pub fn from(sys: &System) -> Option<Vec<MountStats>> {
        match sys.mounts() {
            Ok(mounts) => Some(
                mounts
                    .into_iter()
                    .filter_map(|mount| {
                        if mount.total.as_u64() == 0 {
                            None
                        } else {
                            let used = saturating_sub_bytes(mount.total, mount.avail);
                            Some(MountStats {
                                fs_type: mount.fs_type,
                                mounted_from: mount.fs_mounted_from,
                                mounted_on: mount.fs_mounted_on,
                                used_mb: bytes_to_mb(used),
                                total_mb: bytes_to_mb(mount.total),
                            })
                        }
                    })
                    .collect(),
            ),
            Err(e) => {
                log("Error getting mounts: ", e);
                None
            }
        }
    }
}

/// Network stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStats {
    /// Stats for network interfaces
    interfaces: Option<Vec<NetworkInterfaceStats>>,
    /// Stats for sockets
    sockets: Option<SocketStats>,
}

impl Default for NetworkStats {
    fn default() -> Self {
        NetworkStats {
            interfaces: None,
            sockets: None,
        }
    }
}

impl NetworkStats {
    /// Gets network stats for the provided system.
    pub fn from(sys: &System) -> NetworkStats {
        NetworkStats {
            interfaces: NetworkInterfaceStats::from(sys),
            sockets: SocketStats::from(sys),
        }
    }

    /// Updates stats using the provided system.
    pub fn update(&mut self, sys: &System) {
        self.interfaces = NetworkInterfaceStats::from(sys);
        self.sockets = SocketStats::from(sys);
    }
}

/// Stats for a network interface
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceStats {
    /// The name of the interface
    name: String,
    /// IP addresses associated with this interface
    addresses: Vec<String>,
    /// Total bytes sent via this interface
    sent_bytes: u64,
    /// Total bytes received via this interface
    received_bytes: u64,
    /// Total packets sent via this interface
    sent_packets: u64,
    /// Total packets received via this interface
    received_packets: u64,
    /// Total number of errors that occured while sending data via this interface
    send_errors: u64,
    /// Total number of errors that occured while receiving data via this interface
    receive_errors: u64,
}

impl NetworkInterfaceStats {
    /// Gets a list of network interface stats for the provided system. Returns `None` if an error occurs.
    pub fn from(sys: &System) -> Option<Vec<NetworkInterfaceStats>> {
        match sys.networks() {
            Ok(interfaces) => Some(
                interfaces
                    .into_iter()
                    .filter_map(|(_, interface)| match sys.network_stats(&interface.name) {
                        Ok(stats) => {
                            let addresses = interface
                                .addrs
                                .into_iter()
                                .filter_map(address_to_string)
                                .collect();
                            Some(NetworkInterfaceStats {
                                name: interface.name,
                                addresses,
                                sent_bytes: stats.tx_bytes.as_u64(),
                                received_bytes: stats.rx_bytes.as_u64(),
                                sent_packets: stats.tx_packets,
                                received_packets: stats.rx_packets,
                                send_errors: stats.tx_errors,
                                receive_errors: stats.rx_errors,
                            })
                        }
                        Err(e) => {
                            log(
                                &format!("Error getting stats for interface {}: ", interface.name),
                                e,
                            );
                            None
                        }
                    })
                    .collect(),
            ),
            Err(e) => {
                log("Error getting network interfaces: ", e);
                None
            }
        }
    }
}

/// Stats for sockets
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SocketStats {
    /// Number of TCP sockets in use
    tcp_in_use: usize,
    /// Number of orphaned TCP sockets
    tcp_orphaned: usize,
    /// Number of UDP sockets in use
    udp_in_use: usize,
    /// Number of IPv6 TCP sockets in use
    tcp6_in_use: usize,
    /// Number of IPv6 UDP sockets in use
    udp6_in_use: usize,
}

impl SocketStats {
    /// Gets socket stats for the provided system. Returns `None` if an error occurs.
    pub fn from(sys: &System) -> Option<SocketStats> {
        match sys.socket_stats() {
            Ok(stats) => Some(SocketStats {
                tcp_in_use: stats.tcp_sockets_in_use,
                tcp_orphaned: stats.tcp_sockets_orphaned,
                udp_in_use: stats.udp_sockets_in_use,
                tcp6_in_use: stats.tcp6_sockets_in_use,
                udp6_in_use: stats.udp6_sockets_in_use,
            }),
            Err(e) => {
                log("Error getting socket stats: ", e);
                None
            }
        }
    }
}

/// Logs an error message. If the error is for a stat that isn't supported, logs at info level. Otherwise logs at error level.
fn log(message: &str, e: Error) {
    if e.to_string() == "Not supported" {
        info!("{}{}", message, e);
    } else {
        error!("{}{}", message, e)
    }
}

/// Gets the number of megabytes represented by the provided `ByteSize`.
fn bytes_to_mb(byte_size: ByteSize) -> u64 {
    byte_size.as_u64() / BYTES_PER_MB
}

/// Gets the string representation of a `NetworkAddrs`. Returns `None` if the address is anything other than IPv4 or IPv6.
fn address_to_string(address: NetworkAddrs) -> Option<String> {
    match address.addr {
        IpAddr::V4(x) => Some(x.to_string()),
        IpAddr::V6(x) => Some(x.to_string()),
        _ => None,
    }
}
