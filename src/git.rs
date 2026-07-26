use crate::todo::{Todo, TodoLocation};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use std::collections::HashMap;
use std::io::Error;
use std::path::PathBuf;
use std::process::Command;

pub fn populate_metadata(todo_locations: &Vec<TodoLocation>) -> Result<Vec<Todo>, Error> {
    // Group by file
    let mut grouped: HashMap<PathBuf, Vec<&TodoLocation>> = HashMap::new();
    for todo_location in todo_locations {
        grouped
            .entry(todo_location.path.clone())
            .or_default()
            .push(todo_location);
    }

    let todos: Vec<Todo> = grouped
        .par_iter()
        .flat_map(
            |(file_path, todo_locations)| match get_git_blame_for_file(todo_locations) {
                Ok(todos) => todos,
                Err(e) => {
                    eprintln!(
                        "Warning: couldn't get git blame for {}: {}",
                        file_path.display(),
                        e
                    );
                    vec![]
                }
            },
        )
        .collect();

    Ok(todos)
}

// Batch parses the git blame output for all todo locations in a single file.
pub fn get_git_blame_for_file(todo_locations: &[&TodoLocation]) -> Result<Vec<Todo>, Error> {
    let mut command = build_blame_command(todo_locations)?;
    let output = command.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::other(format!("git blame failed: {}", stderr)));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut line_map: HashMap<usize, String> = HashMap::new();
    let mut commit_data: HashMap<String, (String, DateTime<Utc>)> = HashMap::new();

    const NOT_COMMITTED_YET: &str = "Not Committed Yet";

    let mut current_hash: Option<String> = None;
    let mut current_line_num: Option<usize> = None;
    let mut current_author: String = NOT_COMMITTED_YET.to_string();
    let mut current_timestamp: Option<DateTime<Utc>> = None;

    for line in stdout.lines() {
        // End of entry - commit message is preceded by a tab
        if line.starts_with("\t") {
            if let (Some(hash), Some(timestamp)) = (&current_hash, current_timestamp) {
                commit_data.insert(hash.clone(), (current_author, timestamp));
            }

            if let (Some(hash), Some(line_num)) = (&current_hash, current_line_num) {
                line_map.insert(line_num, hash.clone());
            }

            current_hash = None;
            current_line_num = None;
            current_author = NOT_COMMITTED_YET.to_string();
            current_timestamp = None;
            continue;
        }

        // e.g. ce75ad3e5f0647fe0b1e249db29efd2940d7bcca 12 12 1
        // hash from_line to_line line_count
        // Line count is optional, but if it's more than 1, e.g. 12 12 2
        // means that line 12 and line 13 share the same commit hash.

        if let Some(hash) = line.split_whitespace().next()
            && hash.len() == 40
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                current_line_num = parts[2].parse().ok();
            }
            current_hash = Some(hash.to_string());
        }

        // e.g. author Alex Baker

        if let Some(author) = line.strip_prefix("author ") {
            current_author = author.trim().to_string();
            continue;
        }

        // e.g. author-time 1785057699

        if let Some(time_str) = line.strip_prefix("author-time ") {
            if let Ok(secs) = time_str.trim().parse::<i64>() {
                current_timestamp = DateTime::from_timestamp(secs, 0);
            }
            continue;
        }
    }

    let now = Utc::now();

    let todos: Vec<Todo> = todo_locations
        .iter()
        .map(|loc| {
            let (author, timestamp) = line_map
                .get(&loc.line_number)
                .and_then(|hash| commit_data.get(hash))
                .cloned()
                .unwrap_or_else(|| (NOT_COMMITTED_YET.to_string(), now));
            Todo {
                path: loc.path.clone(),
                line_number: loc.line_number,
                text: loc.text.clone(),
                author: author.clone(),
                timestamp: timestamp.clone(),
                age: now.signed_duration_since(timestamp.clone()),
            }
        })
        .collect();

    Ok(todos)
}

// Builds something like: git blame --porcelain -L1,1 -L31,31 -L512,512 file.rs
// Assumes that all the locations are in the same file.
fn build_blame_command(todo_locations: &[&TodoLocation]) -> Result<Command, Error> {
    if todo_locations.is_empty() {
        return Err(Error::other("Cannot blame an empty list of todo locations"));
    }

    // Assert that all the locations are in the same file
    let file_path = todo_locations[0].path.clone();
    for todo_location in todo_locations {
        if todo_location.path != file_path {
            return Err(Error::other(
                "Locations grouped incorrectly in build_blame_command",
            ));
        }
    }

    let absolute_path = file_path.canonicalize()?;
    let parent_dir = absolute_path
        .parent()
        .ok_or_else(|| Error::other("File path has no parent directory"))?;

    let line_args: Vec<String> = todo_locations
        .iter()
        .flat_map(|loc| {
            vec![
                "-L".to_string(),
                format!("{},{}", loc.line_number, loc.line_number),
            ]
        })
        .collect();

    let mut command = Command::new("git");

    command
        .arg("blame")
        .arg("--porcelain")
        .args(&line_args)
        .arg(&absolute_path)
        .current_dir(parent_dir);

    Ok(command)
}
