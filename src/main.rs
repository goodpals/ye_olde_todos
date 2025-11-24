use chrono::{DateTime, Utc};
use clap::Parser;
use ignore::Walk;
use std::{
    fs::File,
    io::{BufRead, BufReader, Error, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
};
use terminal_size::terminal_size;

mod todo;
use todo::{Todo, TodoLocation};

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
        if line.contains("// TODO:") || line.contains("# TODO:") {
            todos.push(TodoLocation {
                path: path.to_path_buf(),
                line_number: line_number + 1,
                text: line.trim().to_string(),
            });
        }
    }
    Ok(todos)
}

fn populate_metadata(todo_locations: &Vec<TodoLocation>) -> Result<Vec<Todo>, Error> {
    let mut todos = Vec::new();
    for todo_location in todo_locations {
        match get_git_blame(todo_location) {
            Ok(todo) => todos.push(todo),
            Err(e) => eprintln!(
                "Warning: couldn't get git blame for {}: {}",
                todo_location.path.display(),
                e
            ),
        }
    }
    Ok(todos)
}

fn get_git_blame(todo_location: &TodoLocation) -> Result<Todo, Error> {
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
        .current_dir(parent_dir) // Run git from file's directory
        .output()?;

    // Check if git command succeeded
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

    Ok(Todo {
        path: todo_location.path.clone(),
        line_number: todo_location.line_number,
        text: todo_location.text.clone(),
        timestamp,
        author: author.to_string(),
    })
}
