use serde::Serialize;

use crate::{stats::GeneralStats, stats_history::StatsHistory};

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
    y_values: Vec<i32>,
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
        charts.push(ChartContext {
            id: "test-chart".to_string(),
            title: "Test Chart".to_string(),
            x_label: "Time".to_string(),
            y_label: "Amount".to_string(),
            x_values: vec![
                "a long time ago".to_string(),
                "not that long ago".to_string(),
                "just recently".to_string(),
            ],
            y_values: vec![4, 2, 3, 5, 2, 4, 5],
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
            y_values: vec![9, 8, 3, 5, 2, 4, 5],
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
