use clap::Parser;
use ignore::Walk;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    path::{Path, PathBuf},
};
use terminal_size::terminal_size;

mod git;
mod todo;
use git::populate_metadata;
use todo::TodoLocation;

#[derive(Parser)]
#[command(name = "ye_olde_todos")]
#[command(about = "Find and manage TODO comments by age")]
struct Args {
    #[arg(short, long)]
    limit: Option<usize>,
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
}

// TODO: test 2
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
    for todo in todos.iter().take(limit) {
        println!(
            "{}",
            todo.to_string(max_name_length, max_filename_length, terminal_width)
        );
    }
}

fn scan_for_todos(root: &PathBuf) -> Result<Vec<TodoLocation>, Error> {
    let mut todos = Vec::new();
    for entry in Walk::new(root) {
        match entry {
            Ok(entry) => {
                if entry.path().is_file() {
                    todos.extend(scan_file(entry.path()).unwrap());
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
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
