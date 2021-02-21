use crate::stats::AllStats;
use arr_macro::arr;

const STATS_HISTORY_SIZE: usize = 100;

pub struct StatsHistory {
    stats: [Option<AllStats>; STATS_HISTORY_SIZE],
    most_recent_index: usize,
}

impl Default for StatsHistory {
    fn default() -> Self {
        StatsHistory {
            stats: arr![None; 100],
            most_recent_index: 0,
        }
    }
}

impl StatsHistory {
    /// Creates a `StatsHistory`.
    pub fn new() -> StatsHistory {
        StatsHistory::default()
    }

    /// Adds stats to the history.
    /// # Params
    /// * `new_stats` - The stats to add.
    pub fn push(&mut self, new_stats: AllStats) {
        let new_index = match self.get_most_recent_stats() {
            Some(_) => self.get_next_index(),
            None => self.most_recent_index,
        };
        self.most_recent_index = new_index;

        self.stats[self.most_recent_index] = Some(new_stats);
    }

    /// Gets the most recently added stats from the history. Returns `None` if the history is empty.
    pub fn get_most_recent_stats(&self) -> Option<&AllStats> {
        self.stats[self.most_recent_index].as_ref()
    }

    fn get_next_index(&self) -> usize {
        (self.most_recent_index + 1) % (STATS_HISTORY_SIZE - 1)
    }
}

impl<'a> IntoIterator for &'a StatsHistory {
    type Item = &'a AllStats;
    type IntoIter = StatsHistoryIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let starting_index = match self.stats.last() {
            // The array is full, so the oldest stats are in the next index. (Since it wraps around)
            Some(_) => self.get_next_index(),
            // The array is not full, so the oldest stats are at the beginning of the array.
            None => 0,
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

        let result = self.stats_history.stats[self.index].as_ref();
        if self.index == self.stats_history.most_recent_index {
            self.done = true;
        } else {
            self.index = (self.index + 1) % (STATS_HISTORY_SIZE - 1);
        }

        result
    }
}
