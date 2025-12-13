use chrono::{DateTime, Duration, Utc};
use owo_colors::OwoColorize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone)]
pub struct TodoLocation {
    pub path: PathBuf,
    pub line_number: usize,
    pub text: String,
}

#[derive(Serialize)]
pub struct Todo {
    pub path: PathBuf,
    pub line_number: usize,
    pub text: String,
    pub author: String,
    #[serde(serialize_with = "serialize_datetime")]
    pub timestamp: DateTime<Utc>,
    #[serde(serialize_with = "serialize_duration")]
    pub age: Duration,
}

fn serialize_datetime<S>(dt: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&dt.to_rfc3339())
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_i64(duration.num_milliseconds())
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
        let days = self.age.num_days();
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
        let hours = self.age.num_hours();
        if hours > 0 {
            return format!("{} hours", hours);
        }
        let minutes = self.age.num_minutes();
        if minutes > 0 {
            return format!("{} minutes", minutes);
        }
        return format!("{} seconds", self.age.num_seconds());
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
