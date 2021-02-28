use chrono::{DateTime, Local, NaiveDateTime, SecondsFormat, Utc};
use serde::Serialize;

use crate::{
    stats::{GeneralStats, MountStats, NetworkStats},
    stats_history::StatsHistory,
};

const CPU_PER_LOGICAL_CPU_LINE_COLOR_LIGHT_MODE: &str = "#00000044"; // gray
const CPU_PER_LOGICAL_CPU_LINE_COLOR_DARK_MODE: &str = "#ffffff44"; // gray
const CPU_AGGREGATE_LINE_COLOR: &str = "#ffcc00"; // yellow
const CPU_AGGREGATE_FILL_COLOR: &str = "#ffcc0099"; // yellow

const TEMPERATURE_LINE_COLOR: &str = "#990000"; // red
const TEMPERATURE_FILL_COLOR: &str = "#99000099"; // red

const MEM_LINE_COLOR: &str = "#0055ff"; // blue
const MEM_FILL_COLOR: &str = "#0055ff99"; // blue

const SENT_LINE_COLOR: &str = "#44eeaa"; // blue-green
const SENT_FILL_COLOR: &str = "#44eeaa99"; // blue-green
const RECEIVED_LINE_COLOR: &str = "#44ee77"; // green
const RECEIVED_FILL_COLOR: &str = "#44ee7799"; // green

const SEND_ERRORS_LINE_COLOR: &str = "#ff8800"; // yellow-orange
const SEND_ERRORS_FILL_COLOR: &str = "#ff880099"; // yellow-orange
const RECEIVE_ERRORS_LINE_COLOR: &str = "#ff6600"; // orange
const RECEIVE_ERRORS_FILL_COLOR: &str = "#ff660099"; // orange

const TCP_LINE_COLOR: &str = "#44eedd"; // teal
const TCP_FILL_COLOR: &str = "#44eedd99"; // teal
const UDP_LINE_COLOR: &str = "#44bbdd"; // light blue
const UDP_FILL_COLOR: &str = "#44bbdd99"; // light blue

const LOAD_AVERAGE_1_LINE_COLOR: &str = "#ff00ff"; // pink
const LOAD_AVERAGE_1_FILL_COLOR: &str = "#ff00ff99"; // pink
const LOAD_AVERAGE_5_LINE_COLOR: &str = "#bb00ff"; // purple
const LOAD_AVERAGE_5_FILL_COLOR: &str = "#bb00ff99"; // purple
const LOAD_AVERAGE_15_LINE_COLOR: &str = "#7700ff"; // dark purple
const LOAD_AVERAGE_15_FILL_COLOR: &str = "#7700ff99"; // dark purple

#[derive(Serialize)]
pub struct DashboardContext {
    title: String,
    dark_mode: bool,
    charts: Vec<ChartContext>,
    sections: Vec<DashboardSectionContext>,
    last_update_time: String,
}

#[derive(Serialize)]
struct ChartContext {
    /// The id of this chart. Must be unique.
    id: String,
    /// The title of this chart.
    title: String,
    /// The datasets displayed on this chart.
    datasets: Vec<DatasetContext>,
    /// The label for the X axis.
    x_label: String,
    /// The label for the Y axis.
    y_label: String,
    /// Names of the markers on the X axis.
    x_values: Vec<String>,
    /// The lowest possible Y value expected for this chart.
    min_y: f32,
    /// The highest possible Y value expected for this chart.
    max_y: f32,
    /// First line of text to diplay beside the chart.
    accompanying_text_1: String,
    /// Second line of text to diplay beside the chart.
    accompanying_text_2: String,
}

#[derive(Serialize)]
struct DatasetContext {
    /// The name of this dataset.
    name: String,
    /// Color code used for the line.
    line_color_code: String,
    /// Color code used for the area under the line. Only relevant if `fill` is `true`.
    fill_color_code: String,
    /// The values in this dataset.
    values: Vec<f32>,
    /// Whether to fill the area under the line.
    fill: bool,
}

#[derive(Serialize)]
struct DashboardSectionContext {
    name: String,
    stats: Vec<String>,
    subsections: Vec<DashboardSubsectionContext>,
}

#[derive(Serialize)]
struct DashboardSubsectionContext {
    name: String,
    stats: Vec<String>,
}

impl DashboardContext {
    /// Builds a `DashboardContext` from the provided stats history.
    /// # Params
    /// * `stats_history` - The stats history to use to populate the context.
    /// * `dark_mode` - Whether dark mode is enabled or not.
    pub fn from(stats_history: &StatsHistory, dark_mode: bool) -> DashboardContext {
        let title = "Dashboard".to_string();

        let mut sections = Vec::new();
        let most_recent_stats = match stats_history.get_most_recent_stats() {
            Some(x) => x,
            None => {
                return DashboardContext {
                    title,
                    dark_mode,
                    charts: Vec::new(),
                    sections: vec![DashboardSectionContext {
                        name: "No stats yet".to_string(),
                        stats: Vec::new(),
                        subsections: Vec::new(),
                    }],
                    last_update_time: "N/A".to_string(),
                }
            }
        };

        if let Some(x) = build_general_section(&most_recent_stats.general) {
            sections.push(x);
        }
        if let Some(x) = &most_recent_stats.filesystems {
            sections.push(build_filesystems_section(x));
        }
        if let Some(x) = build_network_section(&most_recent_stats.network) {
            sections.push(x);
        }

        let mut charts = Vec::new();
        charts.extend(build_cpu_charts(stats_history, dark_mode));
        charts.push(build_memory_chart(stats_history));
        charts.push(build_load_average_chart(stats_history));
        charts.extend(build_network_charts(stats_history));

        DashboardContext {
            title,
            dark_mode,
            charts,
            sections,
            last_update_time: most_recent_stats
                .collection_time
                .to_rfc3339_opts(SecondsFormat::Millis, true),
        }
    }
}

fn build_general_section(stats: &GeneralStats) -> Option<DashboardSectionContext> {
    let mut stat_strings = Vec::new();
    if let Some(x) = stats.uptime_seconds {
        stat_strings.push(format!("Uptime: {} seconds", x))
    };
    if let Some(x) = stats.boot_timestamp {
        let parsed_time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(x, 0), Utc);
        stat_strings.push(format!(
            "Boot time: {}",
            parsed_time.with_timezone(&Local).to_rfc3339()
        ))
    }

    if stat_strings.is_empty() {
        None
    } else {
        Some(DashboardSectionContext {
            name: "General".to_string(),
            stats: stat_strings,
            subsections: Vec::new(),
        })
    }
}

fn build_filesystems_section(mount_stats: &[MountStats]) -> DashboardSectionContext {
    //TODO
    DashboardSectionContext {
        name: "Filesystems".to_string(),
        stats: Vec::new(),
        subsections: Vec::new(),
    }
}

fn build_network_section(stats: &NetworkStats) -> Option<DashboardSectionContext> {
    None //TODO
}

fn build_cpu_charts(stats_history: &StatsHistory, dark_mode: bool) -> Vec<ChartContext> {
    let mut charts = Vec::new();
    let mut cpu_datasets = Vec::new();
    let mut aggregate_values = Vec::new();
    let mut per_logical_cpu_values = Vec::new();
    let mut temp_values = Vec::new();
    let mut x_values = Vec::new();
    let empty_vec = Vec::new();
    for stats in stats_history.into_iter() {
        aggregate_values.push(stats.cpu.aggregate_load_percent.unwrap_or(0.0));
        per_logical_cpu_values.push(
            stats
                .cpu
                .per_logical_cpu_load_percent
                .as_ref()
                .unwrap_or(&empty_vec),
        );
        temp_values.push(stats.cpu.temp_celsius.unwrap_or(0.0));
        x_values.push(format_time(stats.collection_time));
    }

    let usage_accompanying_text = format!("{:.2}%", aggregate_values.last().unwrap_or(&0.0));

    cpu_datasets.push(DatasetContext {
        name: "Aggregate".to_string(),
        line_color_code: CPU_AGGREGATE_LINE_COLOR.to_string(),
        fill_color_code: CPU_AGGREGATE_FILL_COLOR.to_string(),
        values: aggregate_values,
        fill: true,
    });

    // TODO there's gotta be a better way to do this
    let num_logical_cpus = match per_logical_cpu_values.first() {
        Some(x) => x.len(),
        None => 0,
    };
    let mut per_logical_cpu_values_flipped: Vec<Vec<f32>> = Vec::new();
    for _ in 0..num_logical_cpus {
        per_logical_cpu_values_flipped.push(Vec::new());
    }
    for vec in per_logical_cpu_values {
        for (i, x) in vec.iter().enumerate() {
            per_logical_cpu_values_flipped[i].push(*x);
        }
    }

    let per_logical_cpu_line_color = if dark_mode {
        CPU_PER_LOGICAL_CPU_LINE_COLOR_DARK_MODE
    } else {
        CPU_PER_LOGICAL_CPU_LINE_COLOR_LIGHT_MODE
    };
    for (i, values) in per_logical_cpu_values_flipped.into_iter().enumerate() {
        cpu_datasets.push(DatasetContext {
            name: format!("CPU {}", i),
            line_color_code: per_logical_cpu_line_color.to_string(),
            fill_color_code: "".to_string(),
            values,
            fill: false,
        });
    }

    charts.push(ChartContext {
        id: "cpu-usage-chart".to_string(),
        title: "CPU Usage".to_string(),
        datasets: cpu_datasets,
        x_label: "Time".to_string(),
        y_label: "Usage (%)".to_string(),
        x_values: x_values.clone(),
        min_y: 0.0,
        max_y: 100.0,
        accompanying_text_1: usage_accompanying_text,
        accompanying_text_2: "".to_string(),
    });

    let temp_accompanying_text = format!("{:.2}°C", temp_values.last().unwrap_or(&0.0));
    charts.push(ChartContext {
        id: "cpu-temp-chart".to_string(),
        title: "Temperature".to_string(),
        datasets: vec![DatasetContext {
            name: "Celsius".to_string(),
            line_color_code: TEMPERATURE_LINE_COLOR.to_string(),
            fill_color_code: TEMPERATURE_FILL_COLOR.to_string(),
            values: temp_values,
            fill: true,
        }],
        x_label: "Time".to_string(),
        y_label: "Temperature (C)".to_string(),
        x_values,
        min_y: 0.0,
        max_y: 85.0,
        accompanying_text_1: temp_accompanying_text,
        accompanying_text_2: "".to_string(),
    });

    charts
}

fn build_memory_chart(stats_history: &StatsHistory) -> ChartContext {
    let mut memory_values = Vec::new();
    let mut memory_total_mb = 0;
    let mut x_values = Vec::new();
    for stats in stats_history.into_iter() {
        match &stats.memory {
            Some(x) => {
                if x.total_mb > memory_total_mb {
                    memory_total_mb = x.total_mb;
                }
                memory_values.push(x.used_mb as f32)
            }
            None => memory_values.push(0.0),
        }
        x_values.push(format_time(stats.collection_time));
    }

    let (accompanying_text_1, accompanying_text_2) = {
        match stats_history.get_most_recent_stats() {
            Some(x) => match &x.memory {
                Some(mem) => {
                    let used_pct = ((mem.used_mb as f64) / (mem.total_mb as f64)) * 100.0;
                    (
                        format!("{} / {} MB", mem.used_mb, mem.total_mb),
                        format!("{:.2}%", used_pct),
                    )
                }
                None => ("-- / -- MB".to_string(), "--%".to_string()),
            },
            None => ("-- / -- MB".to_string(), "--%".to_string()),
        }
    };

    ChartContext {
        id: "ram-chart".to_string(),
        title: "Memory Usage".to_string(),
        datasets: vec![DatasetContext {
            name: "MB Used".to_string(),
            line_color_code: MEM_LINE_COLOR.to_string(),
            fill_color_code: MEM_FILL_COLOR.to_string(),
            values: memory_values,
            fill: true,
        }],
        x_label: "Time".to_string(),
        y_label: "Usage (MB)".to_string(),
        x_values,
        min_y: 0.0,
        max_y: memory_total_mb as f32,
        accompanying_text_1,
        accompanying_text_2,
    }
}

fn build_load_average_chart(stats_history: &StatsHistory) -> ChartContext {
    let mut one_min_values = Vec::new();
    let mut five_min_values = Vec::new();
    let mut fifteen_min_values = Vec::new();
    let mut x_values = Vec::new();
    for stats in stats_history.into_iter() {
        match &stats.general.load_averages {
            Some(x) => {
                one_min_values.push(x.one_minute);
                five_min_values.push(x.five_minutes);
                fifteen_min_values.push(x.fifteen_minutes);
            }
            None => {
                one_min_values.push(0.0);
                five_min_values.push(0.0);
                fifteen_min_values.push(0.0);
            }
        }

        x_values.push(format_time(stats.collection_time));
    }

    let accompanying_text = format!(
        "1: {}, 5: {}, 15: {}",
        one_min_values.last().unwrap_or(&0.0),
        five_min_values.last().unwrap_or(&0.0),
        fifteen_min_values.last().unwrap_or(&0.0)
    );
    let datasets = vec![
        DatasetContext {
            name: "1 minute".to_string(),
            line_color_code: LOAD_AVERAGE_1_LINE_COLOR.to_string(),
            fill_color_code: LOAD_AVERAGE_1_FILL_COLOR.to_string(),
            values: one_min_values,
            fill: true,
        },
        DatasetContext {
            name: "5 minutes".to_string(),
            line_color_code: LOAD_AVERAGE_5_LINE_COLOR.to_string(),
            fill_color_code: LOAD_AVERAGE_5_FILL_COLOR.to_string(),
            values: five_min_values,
            fill: true,
        },
        DatasetContext {
            name: "15 minutes".to_string(),
            line_color_code: LOAD_AVERAGE_15_LINE_COLOR.to_string(),
            fill_color_code: LOAD_AVERAGE_15_FILL_COLOR.to_string(),
            values: fifteen_min_values,
            fill: true,
        },
    ];

    ChartContext {
        id: "load-average-chart".to_string(),
        title: "Load Averages".to_string(),
        datasets,
        x_label: "Time".to_string(),
        y_label: "Load average".to_string(),
        x_values,
        min_y: 0.0,
        max_y: 0.0,
        accompanying_text_1: accompanying_text,
        accompanying_text_2: "".to_string(),
    }
}

fn build_network_charts(stats_history: &StatsHistory) -> Vec<ChartContext> {
    let mut sent_mb_values = Vec::new();
    let mut received_mb_values = Vec::new();
    let mut send_errors_values = Vec::new();
    let mut receive_errors_values = Vec::new();
    let mut tcp_sockets_values = Vec::new();
    let mut udp_sockets_values = Vec::new();
    let mut x_values = Vec::new();
    for stats in stats_history.into_iter() {
        match &stats.network.interfaces {
            Some(x) => {
                let mut total_sent_mb = 0.0;
                let mut total_received_mb = 0.0;
                let mut total_send_errors = 0.0;
                let mut total_receive_errors = 0.0;
                for interface_stats in x {
                    total_sent_mb += interface_stats.sent_mb as f32;
                    total_received_mb += interface_stats.received_mb as f32;
                    total_send_errors += interface_stats.send_errors as f32;
                    total_receive_errors += interface_stats.receive_errors as f32;
                }

                sent_mb_values.push(total_sent_mb);
                received_mb_values.push(total_received_mb);
                send_errors_values.push(total_send_errors);
                receive_errors_values.push(total_receive_errors);
            }
            None => {
                sent_mb_values.push(0.0);
                received_mb_values.push(0.0);
                send_errors_values.push(0.0);
                receive_errors_values.push(0.0);
            }
        }

        match &stats.network.sockets {
            Some(x) => {
                tcp_sockets_values.push(x.tcp_in_use as f32);
                udp_sockets_values.push(x.udp_in_use as f32);
            }
            None => {
                tcp_sockets_values.push(0.0);
                udp_sockets_values.push(0.0);
            }
        }

        x_values.push(format_time(stats.collection_time));
    }

    let mut charts = Vec::new();

    let usage_accompanying_text = format!(
        "{} MB sent, {} MB received",
        sent_mb_values.last().unwrap_or(&0.0),
        received_mb_values.last().unwrap_or(&0.0)
    );
    let usage_datasets = vec![
        DatasetContext {
            name: "Sent".to_string(),
            line_color_code: SENT_LINE_COLOR.to_string(),
            fill_color_code: SENT_FILL_COLOR.to_string(),
            values: sent_mb_values,
            fill: true,
        },
        DatasetContext {
            name: "Received".to_string(),
            line_color_code: RECEIVED_LINE_COLOR.to_string(),
            fill_color_code: RECEIVED_FILL_COLOR.to_string(),
            values: received_mb_values,
            fill: true,
        },
    ];

    charts.push(ChartContext {
        id: "network-usage-chart".to_string(),
        title: "Cumulative Network Usage".to_string(),
        datasets: usage_datasets,
        x_label: "Time".to_string(),
        y_label: "Total (MB)".to_string(),
        x_values: x_values.clone(),
        min_y: 0.0,
        max_y: 0.0,
        accompanying_text_1: usage_accompanying_text,
        accompanying_text_2: "".to_string(),
    });

    let errors_accompanying_text = format!(
        "{} send, {} receive",
        send_errors_values.last().unwrap_or(&0.0),
        receive_errors_values.last().unwrap_or(&0.0)
    );
    let errors_datasets = vec![
        DatasetContext {
            name: "Send".to_string(),
            line_color_code: SEND_ERRORS_LINE_COLOR.to_string(),
            fill_color_code: SEND_ERRORS_FILL_COLOR.to_string(),
            values: send_errors_values,
            fill: true,
        },
        DatasetContext {
            name: "Receive".to_string(),
            line_color_code: RECEIVE_ERRORS_LINE_COLOR.to_string(),
            fill_color_code: RECEIVE_ERRORS_FILL_COLOR.to_string(),
            values: receive_errors_values,
            fill: true,
        },
    ];

    charts.push(ChartContext {
        id: "network-errors-chart".to_string(),
        title: "Cumulative Network Errors".to_string(),
        datasets: errors_datasets,
        x_label: "Time".to_string(),
        y_label: "Total errors".to_string(),
        x_values: x_values.clone(),
        min_y: 0.0,
        max_y: 0.0,
        accompanying_text_1: errors_accompanying_text,
        accompanying_text_2: "".to_string(),
    });

    let sockets_accompanying_text = format!(
        "{} TCP, {} UDP",
        tcp_sockets_values.last().unwrap_or(&0.0),
        udp_sockets_values.last().unwrap_or(&0.0)
    );
    let sockets_datasets = vec![
        DatasetContext {
            name: "TCP".to_string(),
            line_color_code: TCP_LINE_COLOR.to_string(),
            fill_color_code: TCP_FILL_COLOR.to_string(),
            values: tcp_sockets_values,
            fill: true,
        },
        DatasetContext {
            name: "UDP".to_string(),
            line_color_code: UDP_LINE_COLOR.to_string(),
            fill_color_code: UDP_FILL_COLOR.to_string(),
            values: udp_sockets_values,
            fill: true,
        },
    ];

    charts.push(ChartContext {
        id: "sockets-chart".to_string(),
        title: "Socket Usage".to_string(),
        datasets: sockets_datasets,
        x_label: "Time".to_string(),
        y_label: "Sockets".to_string(),
        x_values,
        min_y: 0.0,
        max_y: 0.0,
        accompanying_text_1: sockets_accompanying_text,
        accompanying_text_2: "".to_string(),
    });

    charts
}

fn format_time(time: DateTime<Local>) -> String {
    time.format("%I:%M:%S %p").to_string()
}
