use rocket::http::Status;
use rocket_contrib::json::Json;
use systemstat::{Duration, Platform, System};

mod stats;
use stats::*;

#[macro_use]
extern crate rocket;

const DEFAULT_CPU_LOAD_SAMPLE_DURATION: Duration = Duration::from_millis(250);
const MAX_CPU_LOAD_SAMPLE_MS: u16 = 1000;

/// Endpoint to get all the system stats.
#[get("/stats?<cpu_sample_ms>")]
fn get_all_stats(cpu_sample_ms: Option<u16>) -> Result<Json<AllStats>, Status> {
    //TODO use a custom request guard instead?
    let cpu_sample_duration = parse_cpu_sample_ms(cpu_sample_ms)?;
    let sys = System::new();

    Ok(Json(AllStats {
        general: GeneralStats::from(&sys),
        cpu: CpuStats::from(&sys, cpu_sample_duration),
        memory: MemoryStats::from(&sys),
        filesystems: MountStats::from(&sys),
        network: NetworkStats::from(&sys),
    }))
}

/// Endpoint to get general stats.
#[get("/stats/general")]
fn get_general_stats() -> Json<GeneralStats> {
    Json(GeneralStats::from(&System::new()))
}

/// Endpoint to get CPU stats.
#[get("/stats/cpu?<cpu_sample_ms>")]
fn get_cpu_stats(cpu_sample_ms: Option<u16>) -> Result<Json<CpuStats>, Status> {
    //TODO use a custom request guard instead?
    let cpu_sample_duration = parse_cpu_sample_ms(cpu_sample_ms)?;
    Ok(Json(CpuStats::from(&System::new(), cpu_sample_duration)))
}

/// Endpoint to get memory stats.
#[get("/stats/memory")]
fn get_memory_stats() -> Result<Json<MemoryStats>, Status> {
    match MemoryStats::from(&System::new()) {
        Some(x) => Ok(Json(x)),
        None => Err(Status::InternalServerError),
    }
}

/// Endpoint to get filesystem stats.
#[get("/stats/filesystems")]
fn get_filesystem_stats() -> Result<Json<Vec<MountStats>>, Status> {
    match MountStats::from(&System::new()) {
        Some(x) => Ok(Json(x)),
        None => Err(Status::InternalServerError),
    }
}

/// Endpoint to get network stats.
#[get("/stats/network")]
fn get_network_stats() -> Json<NetworkStats> {
    Json(NetworkStats::from(&System::new()))
}

#[launch]
fn rocket() -> rocket::Rocket {
    rocket::ignite().mount(
        "/",
        routes![
            get_all_stats,
            get_general_stats,
            get_cpu_stats,
            get_memory_stats,
            get_filesystem_stats,
            get_network_stats,
        ],
    )
}

/// Parses the provided CPU sample milleseconds into a `Duration`.
///
/// If `cpu_sample_ms` is `None`, `DEFAULT_CPU_LOAD_SAMPLE_DURATION` will be returned.
/// If `cpu_sample_ms` is greater than `MAX_CPU_LOAD_SAMPLE_MS`, `Err` will be returned.
/// Otherwise, a `Duration` built from `cpu_sample_ms` will be returned.
fn parse_cpu_sample_ms(cpu_sample_ms: Option<u16>) -> Result<Duration, Status> {
    match cpu_sample_ms {
        Some(x) => {
            if x > MAX_CPU_LOAD_SAMPLE_MS {
                Err(Status::BadRequest)
            } else {
                Ok(Duration::from_millis(x.into()))
            }
        }
        None => Ok(DEFAULT_CPU_LOAD_SAMPLE_DURATION),
    }
}
