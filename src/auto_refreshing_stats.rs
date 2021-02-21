/*
use std::{thread::Thread, time::Duration};
use systemstat::System;

use crate::stats::AllStats;

/// System stats that refresh periodically.
struct AutoRefreshingStats {
    /// How often the stats should be refreshed.
    refresh_frequency: Duration,
    /// The system to pull stats from.
    system: System,
    /// The thread that handles the refreshing of the stats.
    refresh_thread: Thread,
    /// The system stats.
    stats: AllStats,
}

impl AutoRefreshingStats {
    fn new(system: System, refresh_frequency: Duration) -> AutoRefreshingStats {

        //TODO
    }
}
*/
