# system-stats-dashboard
Provides a simple dashboard for viewing system stats, and an API for retrieving said stats programmatically.

There are 3 levels of stats: "current", "recent", and "persisted". Current and recent stats are all kept in memory, and persisted stats are saved to disk. The recent stats are a subset of the persisted stats.

By default:
* Current stats are updated every 3 seconds.
* Every minute, the last minute of current stats are consolidated and added as a single entry to the list of recent and persisted stats.
* 180 entries (3 hours) are kept in the recent list, and 2 megabytes (~2000 entries, ~33 hours) are kept in the persisted list.
* Persisted stats are stored in `./stats_history`.

# Configuration
Configuration options are located in `Rocket.toml`.
|Name|Default value|Description|
|----|-------------|-----------|
|address|`"0.0.0.0"`|The address to run the server on|
|port|`8000`|The port to run the server on|
|recent_history_size|`180`|The number of entries to keep in recent history|
|consolidation_limit|`20`|The number of entries to collect before consolidating them and writing an entry to recent and persisted stats|
|update_frequency_seconds|`3`|The number of seconds to wait between each stats collection|
|persist_history|`true`|Whether to persist stats to disk or not. If set to `false`, all the config options below are ignored.|
|history_files_directory|`"./stats_history"`|The directory to persist stats to|
|history_files_max_size_bytes|`2_000_000`|The maximum size, in bytes, to allow `history_files_directory` to grow to|

# Endpoints

## Dashboard

### `/dashboard`
Displays current stats, as well as graphs of some recent stats. Defaults to dark mode; add `?dark=false` for light mode.

![dark_dashboard](https://user-images.githubusercontent.com/48834501/111235475-b7458880-85be-11eb-90a0-0c5d3de4d49b.png)

### `/dashboard/history`
Same as `/dashboard`, except for persisted stats.

## API

### GET `/stats`
Returns all the most recently collected stats.

Example response:
```json
{
  "general": {
    "uptimeSeconds": 5239,
    "bootTimestamp": 1615846969,
    "loadAverages": {
      "oneMinute": 0.0,
      "fiveMinutes": 0.01,
      "fifteenMinutes": 0.0
    }
  },
  "cpu": {
    "perLogicalCpuLoadPercent": [
      0.0,
      0.0,
      0.0,
      0.0
    ],
    "aggregateLoadPercent": 0.2450943,
    "tempCelsius": 50.464
  },
  "memory": {
    "usedMb": 52,
    "totalMb": 969
  },
  "filesystems": [
    {
      "fsType": "ext4",
      "mountedFrom": "/dev/root",
      "mountedOn": "/",
      "usedMb": 8208,
      "totalMb": 62699
    }
  ],
  "network": {
    "interfaces": [
      {
        "name": "wlan0",
        "addresses": [
          "192.168.1.100"
        ],
        "sentMb": 1,
        "receivedMb": 1,
        "sentPackets": 4391,
        "receivedPackets": 7024,
        "sendErrors": 0,
        "receiveErrors": 0
      }
    ],
    "sockets": {
      "tcpInUse": 5,
      "tcpOrphaned": 0,
      "udpInUse": 4,
      "tcp6InUse": 4,
      "udp6InUse": 3
    }
  },
  "collectionTime": "2021-03-15T18:50:07.721739139-05:00"
}
```

### GET `/stats/general`
Returns the most recently collected general stats.

Example response:
```json
{
  "uptimeSeconds": 5239,
  "bootTimestamp": 1615846969,
  "loadAverages": {
    "oneMinute": 0.0,
    "fiveMinutes": 0.01,
    "fifteenMinutes": 0.0
  }
}
```

### GET `/stats/cpu`
Returns the most recently collected stats related to the CPU.

Example response:
```json
{
  "perLogicalCpuLoadPercent": [
    0.0,
    0.0,
    0.0,
    0.0
  ],
  "aggregateLoadPercent": 0.2450943,
  "tempCelsius": 50.464
}
```

### GET `/stats/memory`
Returns the most recently collected stats related to memory.

Example response:
```json
{
  "usedMb": 52,
  "totalMb": 969
}
```

### GET `/stats/filesystems`
Returns the most recently collected stats related to filesystems.

Example response:
```json
[
  {
    "fsType": "ext4",
    "mountedFrom": "/dev/root",
    "mountedOn": "/",
    "usedMb": 8208,
    "totalMb": 62699
  }
]
```

### GET `/stats/network`
Returns the most recently collected stats related to the network.

Example response:
```json
{
  "interfaces": [
    {
      "name": "wlan0",
      "addresses": [
        "192.168.1.100"
      ],
      "sentMb": 1,
      "receivedMb": 1,
      "sentPackets": 4391,
      "receivedPackets": 7024,
      "sendErrors": 0,
      "receiveErrors": 0
    }
  ],
  "sockets": {
    "tcpInUse": 5,
    "tcpOrphaned": 0,
    "udpInUse": 4,
    "tcp6InUse": 4,
    "udp6InUse": 3
  }
}
```

# Possible features to add
* Load saved history from disk on startup
* Send emails if certain stats are above/below certain values for a certain amount of time
