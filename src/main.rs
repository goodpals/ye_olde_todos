use clap::Parser;
use ignore::Walk;
use rayon::prelude::*;
use serde_json;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    path::{Path, PathBuf},
};
use terminal_size::terminal_size;

mod git;
mod stats;
mod todo;
use git::populate_metadata;
use stats::{TodoStats, calculate_stats, format_stats};
use todo::{Todo, TodoLocation};

#[derive(Parser)]
#[command(name = "ye_olde_todos")]
#[command(about = "Find and manage TODO comments by age")]
struct Args {
    #[arg(short, long)]
    limit: Option<usize>,
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
    #[arg(long = "no-stats", action = clap::ArgAction::SetTrue)]
    no_stats: bool,
    #[arg(long, action = clap::ArgAction::SetTrue)]
    json: bool,
}

fn main() {
    let args = Args::parse();

    let todo_locations = scan_for_todos(&args.path).unwrap();
    let mut todos = populate_metadata(&todo_locations).unwrap();
    todos.sort_by_key(|todo| todo.timestamp);

    let max_name_length = todos.iter().map(|t| t.author.len()).max().unwrap_or(9) + 1;
    let max_filename_length = todos
        .iter()
        .map(|t| t.filename_with_line_number().len())
        .max()
        .unwrap_or(20)
        .max(15);
    let mut terminal_width = terminal_size().unwrap().0.0 as usize;
    let min_terminal_width = max_name_length + max_filename_length + 30;
    terminal_width = terminal_width.max(min_terminal_width);

    let limit = args.limit.unwrap_or(todos.len());
    let filtered_count = limit.min(todos.len());
    let total_count = todos.len();

    let filtered_todos = &todos[..filtered_count];

    let stats = if !args.no_stats {
        Some(calculate_stats(filtered_todos))
    } else {
        None
    };

    if args.json {
        output_json(filtered_todos, stats, filtered_count, total_count);
    } else {
        if stats.is_some() {
            println!(
                "{}",
                format_stats(&stats.unwrap(), filtered_count, total_count)
            );
        }

        for todo in filtered_todos {
            println!(
                "{}",
                todo.to_string(max_name_length, max_filename_length, terminal_width)
            );
        }
    }
}

fn scan_for_todos(root: &PathBuf) -> Result<Vec<TodoLocation>, Error> {
    let files: Vec<_> = Walk::new(root)
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    let todos: Vec<_> = files
        .par_iter()
        .filter_map(|entry| scan_file(entry.path()).ok())
        .flatten()
        .collect();

    Ok(todos)
}

fn scan_file(path: &Path) -> Result<Vec<TodoLocation>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut todos = Vec::new();
    for (line_number, line) in reader.lines().filter_map(Result::ok).enumerate() {
        if line.contains("// TODO") || line.contains("# TODO") {
            todos.push(TodoLocation {
                path: path.to_path_buf(),
                line_number: line_number + 1,
                text: line.trim().to_string(),
            });
        }
    }
    Ok(todos)
}

fn output_json(
    filtered_todos: &[Todo],
    stats: Option<TodoStats>,
    filtered_count: usize,
    total_count: usize,
) {
    let output = serde_json::json!({
        "stats": stats,
        "todos": filtered_todos,
        "filtered_count": filtered_count,
        "total_count": total_count,
    });
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
