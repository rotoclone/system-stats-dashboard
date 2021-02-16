use std::error::Error;

use heim::{
    host, memory, process, sensors,
    units::{information, ratio, thermodynamic_temperature, time},
};
use tokio_stream::StreamExt;

use std::{thread, time::Duration};
//use systemstat::{saturating_sub_bytes, Duration, Platform, System};
//use sysinfo::{NetworkExt, ProcessorExt, System, SystemExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //systemstat();
    //sysinfo();
    heim().await?;
    Ok(())
}

async fn heim() -> Result<(), Box<dyn Error>> {
    // cpu
    let process = process::current().await?;
    let measurement_1 = process.cpu_usage().await?;
    // Or any other async timer at your choice
    //futures_timer::Delay::new(Duration::from_millis(100)).await;
    thread::sleep(Duration::from_millis(250));
    let measurement_2 = process.cpu_usage().await?;

    println!(
        "CPU usage: {} %",
        (measurement_2 - measurement_1).get::<ratio::percent>()
    );

    // uptime
    let uptime = host::uptime().await?;
    println!("Uptime: {} seconds", uptime.get::<time::second>());

    // memory
    let memory = memory::memory().await?;
    let total_memory = memory.total();
    let used_memory = total_memory - memory.available();
    let memory_usage_pct = used_memory / total_memory; //TODO this doesn't work
    println!(
        "Memory usage: {}/{} MB ({}%)",
        used_memory.get::<information::megabyte>(),
        total_memory.get::<information::megabyte>(),
        memory_usage_pct.get::<ratio::percent>()
    );

    // temperature
    while let Some(x) = sensors::temperatures().next().await {
        let temp = x?;
        println!(
            "{} temperature: {}C",
            temp.label().unwrap_or("Unnamed sensor"),
            temp.current()
                .get::<thermodynamic_temperature::degree_celsius>()
        );
    }

    // disk usage
    //TODO

    Ok(())
}

/*
fn sysinfo() {
    let mut sys = System::new_all();

    // We display the disks:
    println!("=> disk list:");
    for disk in sys.get_disks() {
        println!("{:?}", disk);
    }

    // Network data:
    for (interface_name, data) in sys.get_networks() {
        println!(
            "{}: {}/{} B",
            interface_name,
            data.get_received(),
            data.get_transmitted()
        );
    }

    // Components temperature:
    for component in sys.get_components() {
        println!("{:?}", component);
    }

    // Memory information:
    println!("total memory: {} KB", sys.get_total_memory());
    println!("used memory : {} KB", sys.get_used_memory());
    println!("total swap  : {} KB", sys.get_total_swap());
    println!("used swap   : {} KB", sys.get_used_swap());

    // Processors
    sys.refresh_cpu();
    thread::sleep(Duration::from_secs(1));
    sys.refresh_cpu();
    let processors = sys.get_processors();
    println!("# of processors: {}", processors.len());
    for processor in processors {
        println!(
            "Processor {} usage: {}%",
            processor.get_name(),
            processor.get_cpu_usage()
        );
    }

    // Display system information:
    println!("System name:             {:?}", sys.get_name());
    println!("System kernel version:   {:?}", sys.get_kernel_version());
    println!("System OS version:       {:?}", sys.get_os_version());
    println!("System host name:        {:?}", sys.get_host_name());
    println!("System uptime:           {:?}", sys.get_uptime());
}
*/

/*
fn systemstat() {
    let sys = System::new();

    match sys.mounts() {
        Ok(mounts) => {
            println!("\nMounts:");
            for mount in mounts.iter() {
                println!(
                    "{} ---{}---> {} (available {} of {})",
                    mount.fs_mounted_from,
                    mount.fs_type,
                    mount.fs_mounted_on,
                    mount.avail,
                    mount.total
                );
            }
        }
        Err(x) => println!("\nMounts: error: {}", x),
    }

    match sys.block_device_statistics() {
        Ok(stats) => {
            for blkstats in stats.values() {
                println!("{}: {:?}", blkstats.name, blkstats);
            }
        }
        Err(x) => println!("\nBlock statistics error: {}", x.to_string()),
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

    match sys.battery_life() {
        Ok(battery) => print!(
            "\nBattery: {}%, {}h{}m remaining",
            battery.remaining_capacity * 100.0,
            battery.remaining_time.as_secs() / 3600,
            battery.remaining_time.as_secs() % 60
        ),
        Err(x) => print!("\nBattery: error: {}", x),
    }

    match sys.on_ac_power() {
        Ok(power) => println!(", AC power: {}", power),
        Err(x) => println!(", AC power: error: {}", x),
    }

    match sys.memory() {
        Ok(mem) => println!(
            "\nMemory: {} used / {} ({} bytes) total ({:?})",
            saturating_sub_bytes(mem.total, mem.free),
            mem.total,
            mem.total.as_u64(),
            mem.platform_memory
        ),
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

    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            println!("\nMeasuring CPU load...");
            thread::sleep(Duration::from_secs(1));
            let cpu = cpu.done().unwrap();
            println!(
                "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                cpu.user * 100.0,
                cpu.nice * 100.0,
                cpu.system * 100.0,
                cpu.interrupt * 100.0,
                cpu.idle * 100.0
            );
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
*/
