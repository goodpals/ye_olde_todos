use chrono::{DateTime, Utc};
use owo_colors::OwoColorize;
use std::path::PathBuf;

#[derive(Clone)]
pub struct TodoLocation {
    pub path: PathBuf,
    pub line_number: usize,
    pub text: String,
}

// TODO: test 1
pub struct Todo {
    pub path: PathBuf,
    pub line_number: usize,
    pub text: String,
    pub timestamp: DateTime<Utc>,
    pub author: String,
}

impl Todo {
    fn filename(&self) -> String {
        self.path
            .display()
            .to_string()
            .split("/")
            .last()
            .unwrap()
            .to_string()
    }

    pub fn filename_with_line_number(&self) -> String {
        format!("{}:{}", self.filename(), self.line_number)
    }

    fn age_string(&self) -> String {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.timestamp);
        let days = duration.num_days();
        if days > 0 {
            if days > 364 {
                return format!("{:10}", format!("{} days", days)).red().to_string();
            }
            if days > 60 {
                return format!("{:10}", format!("{} days", days))
                    .yellow()
                    .to_string();
            }
            return format!("{:10}", format!("{} days", days))
                .green()
                .to_string();
        }
        let hours = duration.num_hours();
        if hours > 0 {
            return format!("{} hours", hours);
        }
        let minutes = duration.num_minutes();
        if minutes > 0 {
            return format!("{} minutes", minutes);
        }
        return format!("{} seconds", duration.num_seconds());
    }

    fn truncate_text(&self, max_width: usize) -> String {
        if self.text.len() <= max_width {
            self.text.clone()
        } else if max_width <= 3 {
            "...".to_string()
        } else {
            format!("{}...", &self.text[..max_width - 3])
        }
    }

    pub fn to_string(
        &self,
        max_name_length: usize,
        max_filename_length: usize,
        terminal_width: usize,
    ) -> String {
        let text_width = terminal_width - max_name_length - max_filename_length - 30;

        let plain_filename = self.filename_with_line_number();
        let padded_filename = format!("{:<max_filename_length$}", plain_filename);

        let absolute_path = self.path.canonicalize().unwrap_or(self.path.clone());
        let clickable_filename = format!(
            "\x1b]8;;file://{}\x1b\\{}\x1b]8;;\x1b\\",
            absolute_path.display(),
            padded_filename.blue().to_string()
        );

        format!(
            "{:10} {:max_name_length$} {} {:<text_width$}",
            self.age_string(),
            self.author.to_string(),
            clickable_filename,
            self.truncate_text(text_width).italic().to_string()
        )
    }
}
