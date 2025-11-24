use chrono::{DateTime, Utc};
use clap::Parser;
use ignore::Walk;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(name = "ye_olde_todos")]
#[command(about = "Find and manage TODO comments by age")]
struct Args {
    #[arg(default_value = ".")]
    path: PathBuf,
}

struct TodoLocation {
    path: PathBuf,
    line_number: usize,
    text: String,
}

// TODO: test 1
struct Todo {
    location: TodoLocation,
    timestamp: DateTime<Utc>,
}

// TODO: test 2
fn main() {
    let args = Args::parse();
    println!("Hello, world!");
    println!("Scanning directory: {}", args.path.display());

    let todo_locations = scan_for_todos(&args.path).unwrap();
    for todo_location in todo_locations {
        println!("TODO: {}", todo_location.text);
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
        if line.contains("// TODO:") {
            todos.push(TodoLocation {
                path: path.to_path_buf(),
                line_number: line_number + 1,
                text: line.trim().to_string(),
            });
        }
    }
    Ok(todos)
}

impl TodoLocation {
    fn with_timestamp(self, timestamp: DateTime<Utc>) -> Todo {
        Todo {
            location: self,
            timestamp,
        }
    }
}
