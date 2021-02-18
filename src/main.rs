use std::{io::Error, thread};

use rocket_contrib::json::Json;
use serde::Serialize;
use systemstat::{saturating_sub_bytes, ByteSize, Duration, Platform, System};

#[macro_use]
extern crate rocket;

const BYTES_PER_MB: u64 = 1_000_000;

/// All system stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AllStats {
    /// General system stats
    general: GeneralStats,
    /// CPU stats
    cpu: CpuStats,
    /// Memory stats
    memory: MemoryStats,
    /// Stats for each mounted filesystem
    filesystems: Vec<MountStats>,
    /// Network stats
    network: NetworkStats,
}

/// General system stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GeneralStats {
    /// Number of seconds the system has been running
    uptime_seconds: u64,
    /// One, five, and fifteen-minute load average values for the system
    load_averages: [f32; 3],
}

impl GeneralStats {
    /// Gets general stats for the provided system.
    fn from(sys: &System) -> GeneralStats {
        let uptime_seconds = match sys.uptime() {
            Ok(x) => x.as_secs(),
            Err(e) => {
                error!("Error getting uptime: {}", e);
                0
            }
        };

        let load_averages = match sys.load_average() {
            Ok(x) => [x.one, x.five, x.fifteen],
            Err(e) => {
                log("Error getting load average: ", e);
                [0.0, 0.0, 0.0]
            }
        };

        GeneralStats {
            uptime_seconds,
            load_averages,
        }
    }
}

/// CPU stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CpuStats {
    /// Load percentages for each logical CPU
    per_logical_cpu_load_percent: Vec<f32>,
    /// Load percentage of the CPU as a whole
    aggregate_load_percent: f32,
    /// Temperature of the CPU in degrees Celsius
    temp_celsius: Option<f32>,
}

impl CpuStats {
    /// Gets CPU stats for the provided system.
    /// # Params
    /// * `sys` - The system to get stats from.
    /// * `sample_time` - The amount of time to take to sample CPU load. Note that this function will block the thread it's in for this duration before returning.
    fn from(sys: &System, sample_time: Duration) -> CpuStats {
        let cpu_load = sys.cpu_load();
        let cpu_load_aggregate = sys.cpu_load_aggregate();
        thread::sleep(sample_time);
        let per_logical_cpu_load_percent = match cpu_load {
            Ok(x) => match x.done() {
                Ok(cpus) => cpus.iter().map(|cpu| (1.0 - cpu.idle) * 100.0).collect(),
                Err(e) => {
                    log("Error getting per logical CPU load: ", e);
                    Vec::new()
                }
            },
            Err(e) => {
                log("Error getting per logical CPU load: ", e);
                Vec::new()
            }
        };

        let aggregate_load_percent = match cpu_load_aggregate {
            Ok(x) => match x.done() {
                Ok(cpu) => (1.0 - cpu.idle) * 100.0,
                Err(e) => {
                    log("Error getting aggregate CPU load: ", e);
                    0.0
                }
            },
            Err(e) => {
                log("Error getting aggregate CPU load: ", e);
                0.0
            }
        };

        let temp_celsius = match sys.cpu_temp() {
            Ok(x) => Some(x),
            Err(e) => {
                log("Error getting CPU temperature: ", e);
                None
            }
        };

        CpuStats {
            per_logical_cpu_load_percent,
            aggregate_load_percent,
            temp_celsius,
        }
    }
}

/// Memory stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MemoryStats {
    /// Megabytes of memory used
    used_mb: u64,
    /// Megabytes of memory total
    total_mb: u64,
}

impl MemoryStats {
    /// Gets memory stats for the provided system.
    fn from(sys: &System) -> MemoryStats {
        let (used_mb, total_mb) = match sys.memory() {
            Ok(mem) => {
                let used_mem = saturating_sub_bytes(mem.total, mem.free);
                (bytes_to_mb(used_mem), bytes_to_mb(mem.total))
            }
            Err(e) => {
                log("Error getting memory usage: ", e);
                (0, 0)
            }
        };

        MemoryStats { used_mb, total_mb }
    }
}

/// Stats for a mounted filesystem
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MountStats {
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
    /// Gets a list of mount stats for the provided system. Only mounts with more than 0 bytes of total space are included.
    fn from(sys: &System) -> Vec<MountStats> {
        match sys.mounts() {
            Ok(mounts) => mounts
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
            Err(e) => {
                log("Error getting mounts: ", e);
                Vec::new()
            }
        }
    }
}

/// Network stats
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkStats {
    /// Stats for network interfaces
    interfaces: Vec<NetworkInterfaceStats>,
    /// Stats for sockets
    sockets: SocketStats,
}

impl NetworkStats {
    /// Gets network stats for the provided system.
    fn from(sys: &System) -> NetworkStats {
        NetworkStats {
            interfaces: NetworkInterfaceStats::from(sys),
            sockets: SocketStats::from(sys),
        }
    }
}

/// Stats for a network interface
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NetworkInterfaceStats {
    /// The name of the interface
    name: String,
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
    /// Gets a list of network interface stats for the provided system.
    fn from(sys: &System) -> Vec<NetworkInterfaceStats> {
        //TODO
        vec![]
    }
}

/// Stats for sockets
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SocketStats {
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
    /// Gets socket stats for the provided system.
    fn from(sys: &System) -> SocketStats {
        match sys.socket_stats() {
            Ok(stats) => SocketStats {
                tcp_in_use: stats.tcp_sockets_in_use,
                tcp_orphaned: stats.tcp_sockets_orphaned,
                udp_in_use: stats.udp_sockets_in_use,
                tcp6_in_use: stats.tcp6_sockets_in_use,
                udp6_in_use: stats.udp6_sockets_in_use,
            },
            Err(e) => {
                log("Error getting socket stats: ", e);
                SocketStats {
                    tcp_in_use: 0,
                    tcp_orphaned: 0,
                    udp_in_use: 0,
                    tcp6_in_use: 0,
                    udp6_in_use: 0,
                }
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

/// Endpoint to get all the system stats.
#[get("/stats")]
fn stats() -> Json<AllStats> {
    systemstat();
    let sys = System::new();

    Json(AllStats {
        general: GeneralStats::from(&sys),
        cpu: CpuStats::from(&sys, Duration::from_millis(200)),
        memory: MemoryStats::from(&sys),
        filesystems: MountStats::from(&sys),
        network: NetworkStats::from(&sys),
    })
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![stats])
}

//TODO remove
fn systemstat() {
    let sys = System::new();

    match sys.networks() {
        Ok(netifs) => {
            println!("\nNetworks:");
            for netif in netifs.values() {
                println!("{} ({:?})", netif.name, netif.addrs);
            }
        }
        Err(x) => println!("\nNetworks: error: {}", x),
    }

    match sys.networks() {
        Ok(netifs) => {
            println!("\nNetwork interface statistics:");
            for netif in netifs.values() {
                println!(
                    "{} statistics: ({:?})",
                    netif.name,
                    sys.network_stats(&netif.name)
                );
            }
        }
        Err(x) => println!("\nNetworks: error: {}", x),
    }

    match sys.boot_time() {
        Ok(boot_time) => println!("\nBoot time: {}", boot_time),
        Err(x) => println!("\nBoot time: error: {}", x),
    }
}

/// Gets the number of megabytes represented by the provided `ByteSize`.
fn bytes_to_mb(byte_size: ByteSize) -> u64 {
    byte_size.as_u64() / BYTES_PER_MB
}
