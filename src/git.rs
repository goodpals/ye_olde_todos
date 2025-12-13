use crate::todo::{Todo, TodoLocation};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::io::{Error, ErrorKind};
use std::process::Command;

pub fn populate_metadata(todo_locations: &Vec<TodoLocation>) -> Result<Vec<Todo>, Error> {
    let todos: Vec<_> = todo_locations
        .par_iter()
        .filter_map(|todo_location| match get_git_blame(todo_location) {
            Ok(todo) => Some(todo),
            Err(e) => {
                eprintln!(
                    "Warning: couldn't get git blame for {}: {}",
                    todo_location.path.display(),
                    e
                );
                None
            }
        })
        .collect();

    Ok(todos)
}

pub fn get_git_blame(todo_location: &TodoLocation) -> Result<Todo, Error> {
    let absolute_path = todo_location.path.canonicalize()?;
    let parent_dir = absolute_path.parent().unwrap();

    let output = Command::new("git")
        .arg("blame")
        .arg("-L")
        .arg(format!(
            "{},{}",
            todo_location.line_number, todo_location.line_number
        ))
        .arg(absolute_path.to_str().unwrap())
        .current_dir(parent_dir) // Doesn't work without this!
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::new(
            ErrorKind::Other,
            format!("git blame failed: {}", stderr),
        ));
    }

    let stdout = String::from_utf8(output.stdout).unwrap();
    let line = match stdout.lines().next() {
        Some(l) => l,
        None => return Err(Error::new(ErrorKind::Other, "git blame returned no output")),
    };

    // Parse format: "hash (Author Name YYYY-MM-DD HH:MM:SS +ZZZZ linenum) code..."
    let start = line.find('(').unwrap() + 1;
    let end = line.find(')').unwrap();
    let info = &line[start..end];

    // Split off the line number from the end
    let parts: Vec<&str> = info.rsplitn(2, ' ').collect();
    let datetime_author = parts[1]; // "Author Name YYYY-MM-DD HH:MM:SS +ZZZZ"

    // Split off timezone from the end
    let parts: Vec<&str> = datetime_author.rsplitn(2, ' ').collect();
    let tz = parts[0]; // "+0700"
    let before_tz = parts[1]; // "Author Name YYYY-MM-DD HH:MM:SS"

    // Split off time
    let parts: Vec<&str> = before_tz.rsplitn(2, ' ').collect();
    let time = parts[0]; // "13:03:08"
    let before_time = parts[1]; // "Author Name YYYY-MM-DD"

    // Split off date
    let parts: Vec<&str> = before_time.rsplitn(2, ' ').collect();
    let date = parts[0]; // "2025-11-24"
    let author = parts[1]; // "Author Name"

    // Parse datetime: "2025-11-24 13:03:08 +0700"
    let datetime_str = format!("{} {} {}", date, time, tz);
    let timestamp = DateTime::parse_from_str(&datetime_str, "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .with_timezone(&Utc);

    let now = Utc::now();
    let age = now.signed_duration_since(timestamp);

    Ok(Todo {
        path: todo_location.path.clone(),
        line_number: todo_location.line_number,
        text: todo_location.text.clone(),
        author: author.to_string(),
        timestamp,
        age,
    })
}
