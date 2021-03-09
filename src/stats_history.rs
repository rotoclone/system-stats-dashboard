use systemstat::System;
use thread::JoinHandle;

use crate::stats::*;
use std::{
    fs::{create_dir_all, File},
    io::{BufRead, BufReader, Write},
};
use std::{
    fs::{rename, OpenOptions},
    io,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const CURRENT_HISTORY_FILE_NAME: &str = "current_stats.txt";
const OLD_HISTORY_FILE_NAME: &str = "old_stats.txt";

/// Stats history that updates itself periodically.
pub struct UpdatingStatsHistory {
    /// The thread that handles updating the stats.
    _update_thread: JoinHandle<()>,
    /// The stats history.
    pub stats_history: Arc<Mutex<StatsHistory>>,
}

#[derive(Clone)]
pub enum HistoryPersistenceConfig {
    Disabled,
    Enabled {
        /// The base directory to save the stats history to.
        dir: PathBuf,
        /// The maximum size to allow the saved stats history directory to grow to, in bytes.
        size_limit: u64,
    },
}

impl UpdatingStatsHistory {
    /// Creates an `UpdatingStatsHistory`.
    /// # Params
    /// * `system` - The system to gather stats from.
    /// * `cpu_sample_duration` - The amount of time to take to sample CPU load. Must be less than `update_frequency`.
    /// * `update_frequency` - How often new stats should be gathered. Must be greater than `cpu_sample_duration`.
    /// * `history_size` - The maximum number of entries to keep in the history.
    /// * `consolidation_limit` - The number of times to gather stats before consolidating them and adding them to the history.
    /// * `persistence_config` - Configuration for persisting history to disk.
    pub fn new(
        system: System,
        cpu_sample_duration: Duration,
        update_frequency: Duration,
        history_size: NonZeroUsize,
        consolidation_limit: NonZeroUsize,
        persistence_config: HistoryPersistenceConfig,
    ) -> UpdatingStatsHistory {
        //TODO instead of maintaining this list, keep a single moving average?
        let mut recent_stats = Vec::with_capacity(consolidation_limit.get());
        let shared_stats_history = Arc::new(Mutex::new(StatsHistory::new(history_size)));
        let update_thread_stats_history = Arc::clone(&shared_stats_history);
        let update_thread = thread::spawn(move || loop {
            let new_stats = AllStats::from(&system, cpu_sample_duration);
            recent_stats.push(new_stats.clone());

            if recent_stats.len() >= consolidation_limit.get() {
                let consolidated_stats = consolidate_all_stats(recent_stats);
                if let HistoryPersistenceConfig::Enabled { dir, size_limit } = &persistence_config {
                    if let Err(e) = persist_stats(&consolidated_stats, dir, *size_limit) {
                        //TODO use actual logging once https://github.com/SergioBenitez/Rocket/issues/21 is done
                        println!("Error persisting stats to {:?}: {}", dir, e);
                    }
                }

                {
                    let mut history = update_thread_stats_history.lock().unwrap();
                    history.update_most_recent_stats(consolidated_stats);
                    history.push(new_stats);
                }
                recent_stats = Vec::with_capacity(consolidation_limit.get());
            } else {
                let mut history = update_thread_stats_history.lock().unwrap();
                history.update_most_recent_stats(new_stats);
            }

            thread::sleep(update_frequency - cpu_sample_duration);
        });

        UpdatingStatsHistory {
            _update_thread: update_thread,
            stats_history: shared_stats_history,
        }
    }
}

fn consolidate_all_stats(mut stats_list: Vec<AllStats>) -> AllStats {
    if stats_list.is_empty() {
        panic!("stats_list must not be empty")
    }

    // deal with stats we need to calculate averages for first
    let mut average_one_min_load_average = 0.0;
    let mut average_five_min_load_average = 0.0;
    let mut average_fifteen_min_load_average = 0.0;

    let mut average_per_logical_cpu_loads = Vec::new();
    let mut average_aggregate_cpu_load = 0.0;
    let mut average_temp = 0.0;

    let mut average_mem_used = 0.0;
    let mut max_total_mem = 0;

    let mut average_tcp_used = 0.0;
    let mut average_tcp_orphaned = 0.0;
    let mut average_udp_used = 0.0;
    let mut average_tcp6_used = 0.0;
    let mut average_udp6_used = 0.0;

    for (i, all_stats) in stats_list.iter().enumerate() {
        if let Some(load_averages) = &all_stats.general.load_averages {
            average_one_min_load_average =
                average_one_min_load_average.updated_average(load_averages.one_minute, i + 1);
            average_five_min_load_average =
                average_five_min_load_average.updated_average(load_averages.five_minutes, i + 1);
            average_fifteen_min_load_average = average_fifteen_min_load_average
                .updated_average(load_averages.fifteen_minutes, i + 1);
        }

        if let Some(loads) = &all_stats.cpu.per_logical_cpu_load_percent {
            average_per_logical_cpu_loads.update_averages(loads, i + 1);
        }

        if let Some(aggregate) = &all_stats.cpu.aggregate_load_percent {
            average_aggregate_cpu_load =
                average_aggregate_cpu_load.updated_average(*aggregate, i + 1);
        }

        if let Some(temp) = &all_stats.cpu.temp_celsius {
            average_temp = average_temp.updated_average(*temp, i + 1);
        }

        if let Some(memory_stats) = &all_stats.memory {
            average_mem_used = average_mem_used.updated_average(memory_stats.used_mb as f32, i + 1);
            if memory_stats.total_mb > max_total_mem {
                max_total_mem = memory_stats.total_mb;
            }
        }

        if let Some(socket_stats) = &all_stats.network.sockets {
            average_tcp_used =
                average_tcp_used.updated_average(socket_stats.tcp_in_use as f32, i + 1);
            average_tcp_orphaned =
                average_tcp_orphaned.updated_average(socket_stats.tcp_orphaned as f32, i + 1);
            average_udp_used =
                average_udp_used.updated_average(socket_stats.udp_in_use as f32, i + 1);
            average_tcp6_used =
                average_tcp6_used.updated_average(socket_stats.tcp6_in_use as f32, i + 1);
            average_udp6_used =
                average_udp6_used.updated_average(socket_stats.udp6_in_use as f32, i + 1);
        }
    }

    let last_stats = stats_list.pop().unwrap(); // this should never panic because we won't get to here if stats_list is empty
    let general = GeneralStats {
        uptime_seconds: last_stats.general.uptime_seconds,
        boot_timestamp: last_stats.general.boot_timestamp,
        load_averages: Some(LoadAverages {
            one_minute: average_one_min_load_average,
            five_minutes: average_five_min_load_average,
            fifteen_minutes: average_fifteen_min_load_average,
        }),
    };

    let filesystems = last_stats.filesystems;

    let network = NetworkStats {
        interfaces: last_stats.network.interfaces,
        sockets: Some(SocketStats {
            tcp_in_use: average_tcp_used.round() as usize,
            tcp_orphaned: average_tcp_orphaned.round() as usize,
            udp_in_use: average_udp_used.round() as usize,
            tcp6_in_use: average_tcp6_used.round() as usize,
            udp6_in_use: average_udp6_used.round() as usize,
        }),
    };

    let collection_time = last_stats.collection_time;

    AllStats {
        general,
        cpu: CpuStats {
            per_logical_cpu_load_percent: Some(average_per_logical_cpu_loads),
            aggregate_load_percent: Some(average_aggregate_cpu_load),
            temp_celsius: Some(average_temp),
        },
        memory: Some(MemoryStats {
            used_mb: average_mem_used.round() as u64,
            total_mb: max_total_mem,
        }),
        filesystems,
        network,
        collection_time,
    }
}

fn persist_stats(stats: &AllStats, dir: &Path, dir_size_limit_bytes: u64) -> io::Result<()> {
    if !dir.exists() {
        create_dir_all(dir)?;
    }

    let current_stats_path = dir.join(CURRENT_HISTORY_FILE_NAME);
    let old_stats_path = dir.join(OLD_HISTORY_FILE_NAME);

    // divide size limit by 2 since this swaps between 2 files
    if current_stats_path.exists()
        && current_stats_path.metadata()?.len() >= (dir_size_limit_bytes / 2)
    {
        rename(&current_stats_path, &old_stats_path)?;
    }

    let mut current_stats_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(current_stats_path)?;
    writeln!(current_stats_file, "{}", serde_json::to_string(stats)?)?;

    Ok(())
}

trait MovingAverage<T> {
    /// Updates the average to take into account a new value.
    /// # Params
    /// * `new_value` - The new value to add to the average.
    /// * `n` - The number of values in the dataset (including the new one).
    ///
    /// Returns the updated average.
    fn updated_average(self, new_value: T, n: usize) -> T;
}

impl MovingAverage<f32> for f32 {
    fn updated_average(self, new_value: f32, n: usize) -> f32 {
        self + ((new_value - self) / n as f32)
    }
}

trait MovingAverageCollection<T> {
    /// Updates the averages to take into account a new set of values.
    /// # Params
    /// * `new_values` - The new values to add to the averages. If larger than `self`, `self` will be padded with zeroes to match its size.
    /// * `n` - The number of sets of values in the dataset (including the new ones).
    fn update_averages(&mut self, new_values: &[T], n: usize);
}

impl MovingAverageCollection<f32> for Vec<f32> {
    fn update_averages(&mut self, new_values: &[f32], n: usize) {
        while self.len() < new_values.len() {
            self.push(0.0);
        }

        for (i, new_value) in new_values.iter().enumerate() {
            self[i] = self[i] + ((new_value - self[i]) / n as f32)
        }
    }
}

/// A rolling history of system stats. As new stats are added, the oldest stats will be replaced if the history is full.
pub struct StatsHistory {
    /// The maximum size of the stats list.
    max_size: NonZeroUsize,
    /// The list of stats.
    stats: Vec<AllStats>,
    /// The index of the most recently added stats.
    most_recent_index: usize,
}

impl StatsHistory {
    /// Creates a `StatsHistory`.
    /// # Params
    /// * `max_size` - The maximum number of entries to hold in this history.
    pub fn new(max_size: NonZeroUsize) -> StatsHistory {
        StatsHistory {
            max_size,
            stats: Vec::with_capacity(max_size.get()),
            most_recent_index: 0,
        }
    }

    /// Loads stats history from the provided directory.
    /// # Params
    /// * `dir` - The directory to find persisted stats history files in.
    pub fn load_from(dir: &PathBuf) -> io::Result<StatsHistory> {
        let mut stats = Vec::new();

        let old_stats_path = dir.join(OLD_HISTORY_FILE_NAME);
        let current_stats_path = dir.join(CURRENT_HISTORY_FILE_NAME);

        if old_stats_path.exists() {
            let old_stats_file = File::open(old_stats_path)?;
            for line in BufReader::new(old_stats_file).lines() {
                stats.push(serde_json::from_str(&line?)?);
            }
        }

        if current_stats_path.exists() {
            let current_stats_file = File::open(current_stats_path)?;
            for line in BufReader::new(current_stats_file).lines() {
                stats.push(serde_json::from_str(&line?)?);
            }
        }

        match NonZeroUsize::new(stats.len()) {
            Some(size) => Ok(StatsHistory {
                max_size: size,
                stats,
                most_recent_index: size.get() - 1,
            }),
            None => Ok(StatsHistory::new(NonZeroUsize::new(1).unwrap())),
        }
    }

    /// Adds stats to the history.
    /// # Params
    /// * `new_stats` - The stats to add.
    fn push(&mut self, new_stats: AllStats) {
        if self.stats.len() == self.max_size.get() {
            // The list is full, so we need to replace an existing entry
            self.most_recent_index = self.get_next_index();
            self.update_most_recent_stats(new_stats);
        } else {
            // The list isn't full yet, so we can just add a new entry to the end
            self.stats.push(new_stats);
            self.most_recent_index = self.stats.len() - 1;
        }
    }

    /// Replaces the most recently added stats with the provided stats.
    /// # Params
    /// * `new_stats` - The stats to replace the most recent stats with.
    fn update_most_recent_stats(&mut self, new_stats: AllStats) {
        if self.stats.is_empty() {
            self.push(new_stats);
        } else {
            self.stats[self.most_recent_index] = new_stats;
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
        if self.max_size.get() == 1 {
            0
        } else {
            (self.most_recent_index + 1) % (self.max_size.get() - 1)
        }
    }
}

impl<'a> IntoIterator for &'a StatsHistory {
    type Item = &'a AllStats;
    type IntoIter = StatsHistoryIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let starting_index = if self.stats.len() == self.max_size.get() {
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
        if self.done {
            return None;
        }

        let result = &self.stats_history.stats[self.index];
        if self.index == self.stats_history.most_recent_index {
            self.done = true;
        } else {
            self.index = (self.index + 1) % (self.stats_history.max_size.get() - 1);
        }

        Some(result)
    }
}

//TODO tests
