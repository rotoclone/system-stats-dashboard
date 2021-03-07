use std::num::NonZeroUsize;

use rocket::{http::Status, State};
use rocket_contrib::{json::Json, templates::Template};
use systemstat::{Duration, Platform, System};

mod stats;
use stats::*;

mod stats_history;
use stats_history::*;

mod dashboard_context;
use dashboard_context::*;

#[macro_use]
extern crate rocket;

pub const STATS_HISTORY_SIZE: usize = 180;
pub const STATS_CONSOLIDATION_LIMIT: usize = 20;
pub const STATS_UPDATE_FREQUENCY: Duration = Duration::from_secs(3);
const CPU_LOAD_SAMPLE_DURATION: Duration = Duration::from_millis(500);

const PERSIST_HISTORY_TOGGLE_CONFIG_KEY: &str = "persist_history";
const DEFAULT_PERSIST_HISTORY_TOGGLE: bool = true;

const HISTORY_FILES_DIRECTORY_CONFIG_KEY: &str = "history_files_directory";
const DEFAULT_HISTORY_FILES_DIRECTORY: &str = "./stats_history";

const HISTORY_FILES_DIRECTORY_MAX_SIZE_CONFIG_KEY: &str = "history_files_max_size";
const DEFAULT_HISTORY_FILES_DIRECTORY_MAX_SIZE: u64 = 2_000_000;

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

/// Endpoint to view the dashboard.
#[get("/dashboard?<dark>")]
fn dashboard(stats_history: State<UpdatingStatsHistory>, dark: Option<bool>) -> Template {
    let context = DashboardContext::from(
        &stats_history.stats_history.lock().unwrap(),
        dark.unwrap_or(true),
    );
    Template::render("dashboard", &context)
}

#[launch]
fn rocket() -> rocket::Rocket {
    let mut rocket = rocket::ignite()
        .mount(
            "/",
            routes![
                get_all_stats,
                get_general_stats,
                get_cpu_stats,
                get_memory_stats,
                get_filesystem_stats,
                get_network_stats,
                dashboard,
            ],
        )
        .attach(Template::fairing());

    let history_persistence_enabled = rocket
        .figment()
        .extract_inner(PERSIST_HISTORY_TOGGLE_CONFIG_KEY)
        .unwrap_or(DEFAULT_PERSIST_HISTORY_TOGGLE);
    let persistence_config = if history_persistence_enabled {
        let history_files_dir = rocket
            .figment()
            .extract_inner(HISTORY_FILES_DIRECTORY_CONFIG_KEY)
            .unwrap_or(DEFAULT_HISTORY_FILES_DIRECTORY);
        let history_files_dir_max_size = rocket
            .figment()
            .extract_inner(HISTORY_FILES_DIRECTORY_MAX_SIZE_CONFIG_KEY)
            .unwrap_or(DEFAULT_HISTORY_FILES_DIRECTORY_MAX_SIZE);
        println!("Stats history will be persisted to '{}'", history_files_dir);
        HistoryPersistenceConfig::Enabled {
            dir: history_files_dir.into(),
            size_limit: history_files_dir_max_size,
        }
    } else {
        HistoryPersistenceConfig::Disabled
    };

    rocket = rocket.manage(UpdatingStatsHistory::new(
        System::new(),
        CPU_LOAD_SAMPLE_DURATION,
        STATS_UPDATE_FREQUENCY,
        NonZeroUsize::new(STATS_HISTORY_SIZE).unwrap(),
        NonZeroUsize::new(STATS_CONSOLIDATION_LIMIT).unwrap(),
        persistence_config,
    ));

    rocket
}
