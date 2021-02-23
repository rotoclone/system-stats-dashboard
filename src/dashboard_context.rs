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
    id: String,
    title: String,
    x_label: String,
    y_label: String,
    x_values: Vec<String>,
    y_values: Vec<f32>,
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

        let mut charts = Vec::new();
        let stats_update_seconds = STATS_UPDATE_FREQUENCY.as_secs().try_into().unwrap();
        let x_values = (0..=STATS_HISTORY_SIZE * stats_update_seconds)
            .rev()
            .step_by(stats_update_seconds)
            .map(|x| x.to_string())
            .collect();
        let y_values = stats_history
            .into_iter()
            .map(|stats| match stats.cpu.aggregate_load_percent {
                Some(x) => x,
                None => 0.0,
            })
            .collect();
        charts.push(ChartContext {
            id: "cpu-chart".to_string(),
            title: "CPU Usage".to_string(),
            x_label: "Seconds ago".to_string(),
            y_label: "Usage (%)".to_string(),
            x_values,
            y_values,
        });
        charts.push(ChartContext {
            id: "test-chart-2".to_string(),
            title: "Another Test Chart".to_string(),
            x_label: "Time".to_string(),
            y_label: "Amount".to_string(),
            x_values: vec![
                "a long time ago".to_string(),
                "not that long ago".to_string(),
                "just recently".to_string(),
                "right now".to_string(),
            ],
            y_values: vec![9.0, 8.0, 3.5, 5.1, 2.0, 4.0, 5.0],
        });

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
        //TODO

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
