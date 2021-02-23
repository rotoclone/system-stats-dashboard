use systemstat::System;
use thread::JoinHandle;

use crate::stats::AllStats;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

/// Stats history that updates itself periodically.
pub struct UpdatingStatsHistory {
    /// The thread that handles updating the stats.
    _update_thread: JoinHandle<()>,
    /// The stats history.
    pub stats_history: Arc<Mutex<StatsHistory>>,
}

impl UpdatingStatsHistory {
    /// Creates an `UpdatingStatsHistory`.
    /// # Params
    /// * `system` - The system to gather stats from.
    /// * `cpu_sample_duration` - The amount of time to take to sample CPU load.
    /// * `update_frequency` - How often new stats should be gathered.
    /// * `stats_history` - The stats history to update.
    pub fn new(
        system: System,
        cpu_sample_duration: Duration,
        update_frequency: Duration,
        stats_history: StatsHistory,
    ) -> UpdatingStatsHistory {
        let shared_stats_history = Arc::new(Mutex::new(stats_history));
        let update_thread_stats_history = Arc::clone(&shared_stats_history);
        let update_thread = thread::spawn(move || loop {
            let new_stats = AllStats::from(&system, cpu_sample_duration);
            {
                // This needs to be in its own block so the mutex is unlocked before the thread::sleep
                let mut history = update_thread_stats_history.lock().unwrap();
                history.push(new_stats);
            }
            thread::sleep(update_frequency);
        });

        UpdatingStatsHistory {
            _update_thread: update_thread,
            stats_history: shared_stats_history,
        }
    }
}

/// A rolling history of system stats. As new stats are added, the oldest stats will be replaced if the history is full.
pub struct StatsHistory {
    /// The maximum size of the stats list.
    max_size: usize,
    /// The list of stats.
    stats: Vec<AllStats>,
    /// The index of the most recently added stats.
    most_recent_index: usize,
}

impl StatsHistory {
    /// Creates a `StatsHistory`.
    /// # Params
    /// * `max_size` - The maximum number of entries to hold in this history.
    pub fn new(max_size: usize) -> StatsHistory {
        StatsHistory {
            max_size,
            stats: Vec::with_capacity(max_size),
            most_recent_index: 0,
        }
    }

    /// Adds stats to the history.
    /// # Params
    /// * `new_stats` - The stats to add.
    fn push(&mut self, new_stats: AllStats) {
        if self.stats.len() == self.max_size {
            // The list is full, so we need to replace an existing entry
            self.most_recent_index = self.get_next_index();
            self.stats[self.most_recent_index] = new_stats;
        } else {
            // The list isn't full yet, so we can just add a new entry to the end
            self.stats.push(new_stats);
            self.most_recent_index = self.stats.len() - 1;
        }
    }

    /// Gets the most recently added stats from the history. Returns `None` if the history is empty.
    pub fn get_most_recent_stats(&self) -> Option<&AllStats> {
        if self.stats.is_empty() {
            None
        } else {
            Some(&self.stats[self.most_recent_index])
        }
    }

    fn get_next_index(&self) -> usize {
        (self.most_recent_index + 1) % (self.max_size - 1)
    }
}

impl<'a> IntoIterator for &'a StatsHistory {
    type Item = &'a AllStats;
    type IntoIter = StatsHistoryIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let starting_index = if self.stats.len() == self.max_size {
            // The array is full, so the oldest stats are in the next index. (Since it wraps around)
            self.get_next_index()
        } else {
            // The array is not full, so the oldest stats are at the beginning of the array.
            0
        };

        StatsHistoryIterator {
            stats_history: self,
            index: starting_index,
            done: false,
        }
    }
}

pub struct StatsHistoryIterator<'a> {
    stats_history: &'a StatsHistory,
    index: usize,
    done: bool,
}

impl<'a> Iterator for StatsHistoryIterator<'a> {
    type Item = &'a AllStats;

    fn next(&mut self) -> Option<Self::Item> {
        /*
        println!(
            "most recent index: {}, size: {}, index: {}, done: {}",
            self.stats_history.most_recent_index,
            self.stats_history.stats.len(),
            self.index,
            self.done
        );*/
 //TODO remove
        if self.done {
            return None;
        }

        let result = &self.stats_history.stats[self.index];
        if self.index == self.stats_history.most_recent_index {
            self.done = true;
        } else {
            self.index = (self.index + 1) % (self.stats_history.max_size - 1);
        }

        Some(result)
    }
}

//TODO tests
