use std::convert::TryInto;

use serde::Serialize;

use crate::{
    stats::GeneralStats, stats_history::StatsHistory, STATS_HISTORY_SIZE, STATS_UPDATE_FREQUENCY,
};

#[derive(Serialize)]
pub struct DashboardContext {
    title: String,
    charts: Vec<ChartContext>,
    sections: Vec<DashboardSectionContext>,
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
    /// The highest possible Y value expected for this chart.
    max_y: f32,
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
    pub fn from(stats_history: &StatsHistory) -> DashboardContext {
        let title = "Dashboard".to_string();

        let mut sections = Vec::new();
        let most_recent_stats = match stats_history.get_most_recent_stats() {
            Some(x) => x,
            None => {
                return DashboardContext {
                    title,
                    charts: Vec::new(),
                    sections: vec![DashboardSectionContext {
                        name: "No stats yet".to_string(),
                        stats: Vec::new(),
                        subsections: Vec::new(),
                    }],
                }
            }
        };

        if let Some(x) = build_general_section(&most_recent_stats.general) {
            sections.push(x);
        }
        //TODO more sections

        let mut charts = Vec::new();
        charts.extend(build_cpu_charts(stats_history));
        charts.push(build_memory_chart(stats_history));
        //TODO more charts

        DashboardContext {
            title,
            charts,
            sections,
        }
    }
}

fn build_general_section(stats: &GeneralStats) -> Option<DashboardSectionContext> {
    let mut stat_strings = Vec::new();
    if let Some(x) = stats.uptime_seconds {
        stat_strings.push(format!("Uptime: {} seconds", x))
    };

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

fn build_cpu_charts(stats_history: &StatsHistory) -> Vec<ChartContext> {
    let x_values: Vec<String> = build_x_values();

    let mut charts = Vec::new();
    let mut cpu_datasets = Vec::new();
    let mut aggregate_values = Vec::new();
    let mut per_logical_cpu_values = Vec::new();
    let mut temp_values = Vec::new();
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
    }
    pad_vec(&mut aggregate_values, 0.0, x_values.len());
    pad_vec(&mut temp_values, 0.0, x_values.len());

    cpu_datasets.push(DatasetContext {
        name: "Aggregate".to_string(),
        line_color_code: "#000000".to_string(),
        fill_color_code: "#00995599".to_string(),
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

    for (i, mut values) in per_logical_cpu_values_flipped.into_iter().enumerate() {
        pad_vec(&mut values, 0.0, x_values.len());
        cpu_datasets.push(DatasetContext {
            name: format!("CPU {}", i),
            line_color_code: "#00000044".to_string(),
            fill_color_code: "".to_string(),
            values,
            fill: false,
        });
    }

    charts.push(ChartContext {
        id: "cpu-usage-chart".to_string(),
        title: "CPU Usage".to_string(),
        datasets: cpu_datasets,
        x_label: "Seconds ago".to_string(),
        y_label: "Usage (%)".to_string(),
        x_values: x_values.clone(),
        max_y: 100.0,
    });

    charts.push(ChartContext {
        id: "cpu-temp-chart".to_string(),
        title: "Temperature".to_string(),
        datasets: vec![DatasetContext {
            name: "Celsius".to_string(),
            line_color_code: "#000000".to_string(),
            fill_color_code: "#aa000099".to_string(),
            values: temp_values,
            fill: true,
        }],
        x_label: "Seconds ago".to_string(),
        y_label: "Temperature (C)".to_string(),
        x_values,
        max_y: 0.0,
    });

    charts
}

fn build_memory_chart(stats_history: &StatsHistory) -> ChartContext {
    let x_values: Vec<String> = build_x_values();

    let mut memory_values = Vec::new();
    let mut memory_total_mb = 0;
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
    }
    // TODO the first memory value is 0, even when the history is full
    pad_vec(&mut memory_values, 0.0, x_values.len());

    ChartContext {
        id: "ram-chart".to_string(),
        title: "Memory Usage".to_string(),
        datasets: vec![DatasetContext {
            name: "MB Used".to_string(),
            line_color_code: "#000000".to_string(),
            fill_color_code: "#0055ff99".to_string(),
            values: memory_values,
            fill: true,
        }],
        x_label: "Seconds ago".to_string(),
        y_label: "Usage (MB)".to_string(),
        x_values,
        max_y: memory_total_mb as f32,
    }
}

fn build_x_values() -> Vec<String> {
    let stats_update_seconds = STATS_UPDATE_FREQUENCY.as_secs().try_into().unwrap();
    (0..=(STATS_HISTORY_SIZE * stats_update_seconds) - stats_update_seconds)
        .rev()
        .step_by(stats_update_seconds)
        .map(|x| x.to_string())
        .collect()
}

fn pad_vec<T: Copy>(vec: &mut Vec<T>, pad_value: T, target_size: usize) {
    while vec.len() < target_size {
        vec.insert(0, pad_value);
    }
}
