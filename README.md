# system-stats-api
Provides a simple dashboard for viewing system stats, and an API for retrieving said stats programmatically.

There are 3 levels of stats: "current", "recent", and "persisted". Current and recent stats are all kept in memory, and persisted stats are saved to disk periodically. The recent stats are a subset of the persisted stats.

By default:
* Current stats are updated every 3 seconds.
* Every minute, the last minute of current stats are consolidated and added as a single entry to the list of recent and persisted stats.
* 180 entries (3 hours) are kept in the recent list, and 2 megabytes (~2000 entries, ~33 hours) are kept in the persisted list.
* Persisted stats are stored in `./stats_history`.

# Endpoints

## Dashboard

### `/dashboard`
Displays current stats, as well as graphs of some recent stats. Defaults to dark mode; add `?dark=false` for light mode.

![dark_dashboard](https://user-images.githubusercontent.com/48834501/111235475-b7458880-85be-11eb-90a0-0c5d3de4d49b.png)

### `/dashboard/history`
Same as `/dashboard`, except for persisted stats.

![dark_history](https://user-images.githubusercontent.com/48834501/111235631-0be90380-85bf-11eb-8d27-e38435538b70.png)

## API

### GET `/stats`
Returns all the most recently collected stats.
```json
TODO
```

### GET `/stats/general`
Returns the most recently collected general stats.
```json
TODO
```

### GET `/stats/cpu`
Returns the most recently collected stats related to the CPU.
```json
TODO
```

### GET `/stats/memory`
Returns the most recently collected stats related to memory.
```json
TODO
```

### GET `/stats/filesystems`
Returns the most recently collected stats related to filesystems.
```json
TODO
```

### GET `/stats/network`
Returns the most recently collected stats related to the network.
```json
TODO
```

# Possible features to add
* Load saved history from disk on startup
* Send emails if certain stats are above/below certain values for a certain amount of time
