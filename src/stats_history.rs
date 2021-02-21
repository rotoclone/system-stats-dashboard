use crate::stats::AllStats;

/// A rolling history of system stats. As new stats are added, the oldest stats will be replaced if the history is full.
pub struct StatsHistory {
    stats: Vec<AllStats>,
    max_size: usize,
    most_recent_index: usize,
}

impl StatsHistory {
    /// Creates a `StatsHistory`.
    /// # Params
    /// * `max_size` - The maximum number of entries to hold in this history.
    pub fn new(max_size: usize) -> StatsHistory {
        StatsHistory {
            stats: Vec::with_capacity(max_size),
            max_size,
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
