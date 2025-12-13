use crate::todo::Todo;
use owo_colors::OwoColorize;
use serde::Serialize;

#[derive(Serialize)]
pub struct TodoStats {
    pub mean_age_days: f64,
    pub median_age_days: i64,
}

pub fn calculate_stats(todos: &[Todo]) -> TodoStats {
    let mut ages: Vec<i64> = todos.iter().map(|todo| todo.age.num_days()).collect();
    ages.sort();

    let mean_age = match ages.as_slice() {
        [] => 0.0,
        [single] => *single as f64,
        ages => ages.iter().sum::<i64>() as f64 / ages.len() as f64,
    };

    let median_age = match ages.as_slice() {
        [] => 0,
        [single] => *single,
        ages => {
            let mid = ages.len() / 2;
            match ages.len() % 2 {
                0 => (ages[mid - 1] + ages[mid]) / 2,
                _ => ages[mid],
            }
        }
    };

    TodoStats {
        mean_age_days: mean_age,
        median_age_days: median_age,
    }
}

fn format_value(value: &str) -> String {
    format!("{}", value).green().bold().to_string()
}

pub fn format_stats(stats: &TodoStats, filtered_count: usize, total_count: usize) -> String {
    let count_value = format_value(&format!("{}/{}", filtered_count, total_count));
    let count_text = format!("Todos: {}", count_value);

    let mean_value = format_value(&format!("{:.1} days", stats.mean_age_days));
    let mean_text = format!("Mean: {}", mean_value);

    let median_value = format_value(&format!("{} days", stats.median_age_days));
    let median_text = format!("Median: {}", median_value);

    format!("{}  ||  {}  ||  {}\n", count_text, mean_text, median_text)
}
