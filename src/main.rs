use std::thread;

use systemstat::{saturating_sub_bytes, ByteSize, Duration, Platform, System};

fn main() {
    systemstat();
}

fn systemstat() {
    let sys = System::new();

    match sys.mounts() {
        Ok(mounts) => {
            println!("\nMounts:");
            for mount in mounts.iter() {
                if mount.total.as_u64() != 0 {
                    let used = saturating_sub_bytes(mount.total, mount.avail);
                    let used_pct = (used.as_u64() as f64 / mount.total.as_u64() as f64) * 100.0;
                    println!(
                        "{} ({}) at {}: {}/{} available ({}% used)",
                        mount.fs_mounted_from,
                        mount.fs_type,
                        mount.fs_mounted_on,
                        mount.avail,
                        mount.total,
                        used_pct,
                    );
                }
            }
        }
        Err(x) => println!("\nMounts: error: {}", x),
    }

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

    match sys.memory() {
        Ok(mem) => {
            let used_mem = saturating_sub_bytes(mem.total, mem.free);
            let used_pct = (used_mem.as_u64() as f64 / mem.total.as_u64() as f64) * 100.0;
            println!(
                "\nMemory: {}/{} MB used ({}%)",
                bytes_to_mb(used_mem),
                bytes_to_mb(mem.total),
                used_pct,
            )
        }
        Err(x) => println!("\nMemory: error: {}", x),
    }

    match sys.load_average() {
        Ok(loadavg) => println!(
            "\nLoad average: {} {} {}",
            loadavg.one, loadavg.five, loadavg.fifteen
        ),
        Err(x) => println!("\nLoad average: error: {}", x),
    }

    match sys.uptime() {
        Ok(uptime) => println!("\nUptime: {:?}", uptime),
        Err(x) => println!("\nUptime: error: {}", x),
    }

    match sys.boot_time() {
        Ok(boot_time) => println!("\nBoot time: {}", boot_time),
        Err(x) => println!("\nBoot time: error: {}", x),
    }

    let cpu_load = sys.cpu_load();
    let cpu_load_aggregate = sys.cpu_load_aggregate();
    println!("\nMeasuring CPU load...");
    thread::sleep(Duration::from_millis(200));
    match cpu_load {
        Ok(cpus) => {
            for (i, cpu) in cpus.done().unwrap().iter().enumerate() {
                println!("CPU {} load: {}%", i, (1.0 - cpu.idle) * 100.0);
            }
        }
        Err(x) => println!("\nCPU load: error: {}", x),
    }

    match cpu_load_aggregate {
        Ok(cpu) => {
            let cpu = cpu.done().unwrap();
            println!("Total CPU load: {}%", (1.0 - cpu.idle) * 100.0);
        }
        Err(x) => println!("\nCPU load: error: {}", x),
    }

    match sys.cpu_temp() {
        Ok(cpu_temp) => println!("\nCPU temp: {}", cpu_temp),
        Err(x) => println!("\nCPU temp: {}", x),
    }

    match sys.socket_stats() {
        Ok(stats) => println!("\nSystem socket statistics: {:?}", stats),
        Err(x) => println!("\nSystem socket statistics: error: {}", x.to_string()),
    }
}

pub fn bytes_to_mb(byte_size: ByteSize) -> u64 {
    byte_size.as_u64() / 1_000_000u64
}
