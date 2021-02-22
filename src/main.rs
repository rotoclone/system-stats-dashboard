use rocket::{http::Status, State};
use rocket_contrib::json::Json;
use systemstat::{Duration, Platform, System};

mod stats;
use stats::*;

mod stats_history;
use stats_history::*;

#[macro_use]
extern crate rocket;

const STATS_HISTORY_SIZE: usize = 100;
const STATS_UPDATE_FREQUENCY: Duration = Duration::from_secs(3);
const CPU_LOAD_SAMPLE_DURATION: Duration = Duration::from_millis(250);

/// Endpoint to get all the system stats.
#[get("/stats")]
fn get_all_stats(stats_history: State<UpdatingStatsHistory>) -> Result<Json<AllStats>, Status> {
    match stats_history
        .stats_history
        .lock()
        .unwrap()
        .get_most_recent_stats()
    {
        Some(x) => Ok(Json((*x).clone())),
        None => Err(Status::InternalServerError),
    }
}

/// Endpoint to get general stats.
#[get("/stats/general")]
fn get_general_stats(
    stats_history: State<UpdatingStatsHistory>,
) -> Result<Json<GeneralStats>, Status> {
    match stats_history
        .stats_history
        .lock()
        .unwrap()
        .get_most_recent_stats()
    {
        Some(x) => Ok(Json((*x).general.clone())),
        None => Err(Status::InternalServerError),
    }
}

/// Endpoint to get CPU stats.
#[get("/stats/cpu")]
fn get_cpu_stats(stats_history: State<UpdatingStatsHistory>) -> Result<Json<CpuStats>, Status> {
    match stats_history
        .stats_history
        .lock()
        .unwrap()
        .get_most_recent_stats()
    {
        Some(x) => Ok(Json((*x).cpu.clone())),
        None => Err(Status::InternalServerError),
    }
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
    rocket::ignite()
        .mount(
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
        .manage(UpdatingStatsHistory::new(
            System::new(),
            CPU_LOAD_SAMPLE_DURATION,
            STATS_UPDATE_FREQUENCY,
            StatsHistory::new(STATS_HISTORY_SIZE),
        ))
}
