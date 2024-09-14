use clap::Parser;
use regex::Regex;
use std::path::{PathBuf, Path};
use std::fs;
use std::io::{self, BufReader, BufRead};
use colored::Colorize;
use std::fs::File;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "PATTERN")]
    exclude: Option<String>,

    #[arg(short, long, default_value_t = false)]
    all: bool,

    #[arg(short = 'f', long = "filter", value_name = "PATTERN", num_args = 1.., value_delimiter = ' ')]
    filter: Vec<String>,

    #[arg(short = 'd', long, value_name = "DIRECTORY", action = clap::ArgAction::Append)]
    dir: Vec<String>,

    #[arg(value_name = "DIRECTORY", num_args = 0..)]
    additional_dirs: Vec<PathBuf>,

    #[arg(short = 'c', long = "content", help = "Search for content within files")]
    content: bool,

    #[arg(short = 'p', long = "Parameter-show", default_value_t = false)]
    parameter_show: bool,
}

fn main() {
    let args = Args::parse();
    
    let filter_regexes: Vec<Regex> = args.filter
        .iter()
        .filter_map(|pattern| Regex::new(&format!("^{}$", pattern.replace("*", ".*"))).ok())
        .collect();

    let mut directories: Vec<PathBuf> = args.dir.iter().map(PathBuf::from).collect();
    directories.extend(args.additional_dirs.clone());

    // If no directories are specified, use the current directory
    if directories.is_empty() {
        directories.push(PathBuf::from("."));
    }

    let mut all_files = Vec::new();
    let mut all_permission_denied_dirs = Vec::new();
    let mut other_error_occurred = false;
    let mut error_messages = String::new();

    for dir in &directories {
        let (files, perm_denied_dirs, other_error, err_msg) = search_files(dir, &args, &filter_regexes);
        all_files.extend(files);
        all_permission_denied_dirs.extend(perm_denied_dirs);
        other_error_occurred |= other_error;
        if !err_msg.is_empty() {
            error_messages.push_str(&err_msg);
            error_messages.push('\n');
        }
    }

    display_results(&args, &directories, all_files, all_permission_denied_dirs, other_error_occurred, error_messages);
}

fn search_content(file_path: &Path, filter_regexes: &[Regex]) -> io::Result<bool> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if filter_regexes.iter().any(|re| re.is_match(&line)) {
            return Ok(true);
        }
        
    }

    Ok(false)
}

fn search_files(dir: &Path, args: &Args, filter_regexes: &[Regex]) -> (Vec<String>, Vec<String>, bool, String) {
    let mut files = Vec::new();
    let mut permission_denied_dirs = Vec::new();
    let mut other_error_occurred = false;
    let mut error_message = String::new();

    // Check if the path is a directory
    match dir.metadata() {
        Ok(metadata) => {
            if !metadata.is_dir() {
                other_error_occurred = true;
                error_message = format!("Error: {} is not a directory", dir.display());
                return (files, permission_denied_dirs, other_error_occurred, error_message);
            }
        },
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                permission_denied_dirs.push(dir.to_string_lossy().into_owned());
                return (files, permission_denied_dirs, other_error_occurred, error_message);
            } else {
                other_error_occurred = true;
                error_message = format!("Error accessing {}: {}", dir.display(), e);
                return (files, permission_denied_dirs, other_error_occurred, error_message);
            }
        }
    }

    let exclude_regex = args.exclude.as_ref()
        .and_then(|pattern| Regex::new(&format!("^{}$", pattern.replace("*", ".*"))).ok());

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            if e.kind() == io::ErrorKind::PermissionDenied {
                permission_denied_dirs.push(dir.to_string_lossy().into_owned());
                return (files, permission_denied_dirs, other_error_occurred, error_message);
            } else {
                other_error_occurred = true;
                error_message = format!("Error reading directory {}: {}", dir.display(), e);
                return (files, permission_denied_dirs, other_error_occurred, error_message);
            }
        }
    };

    for entry in read_dir {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if path.is_dir() {
                    let (mut sub_files, mut sub_perm_denied, sub_error, sub_err_msg) = search_files(&path, args, filter_regexes);
                    files.append(&mut sub_files);
                    permission_denied_dirs.append(&mut sub_perm_denied);
                    other_error_occurred |= sub_error;
                    if !sub_err_msg.is_empty() {
                        error_message.push_str(&sub_err_msg);
                        error_message.push('\n');
                    }
                } else {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let full_path = path.to_string_lossy().into_owned();
                        
                        let name_matches = (args.all || !file_name.starts_with('.')) &&
                            (filter_regexes.is_empty() || filter_regexes.iter().any(|re| re.is_match(file_name))) &&
                            exclude_regex.as_ref().map_or(true, |re| !re.is_match(file_name));

                        let content_matches = if args.content {
                            match search_content(&path, filter_regexes) {
                                Ok(matches) => matches,
                                Err(e) => {
                                    other_error_occurred = true;
                                    error_message.push_str(&format!("Error reading file {}: {}\n", path.display(), e));
                                    false
                                }
                            }
                        } else {
                            false
                        };

                        if name_matches || content_matches {
                            files.push(full_path);
                        }
                    }
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::PermissionDenied {
                    permission_denied_dirs.push(dir.to_string_lossy().into_owned());
                } else {
                    other_error_occurred = true;
                    error_message.push_str(&format!("Error accessing entry: {}\n", e));
                }
            }
        }
    }

    (files, permission_denied_dirs, other_error_occurred, error_message)
}

fn display_results(args: &Args, directories: &[PathBuf], files: Vec<String>, permission_denied_dirs: Vec<String>, other_error_occurred: bool, error_messages: String) {
    if args.parameter_show {
        println!("\n{}", "Search Parameters:".bold());
        println!("  Exclude pattern: {}", args.exclude.as_deref().unwrap_or("None"));
        println!("  Include hidden files: {}", args.all);
        
        println!("  Filter patterns:");
        if args.filter.is_empty() {
            println!("    None");
        } else {
            for pattern in &args.filter {
                println!("    - {}", pattern);
            }
        }
        
        println!("  Directories searched:");
        for dir in directories {
            println!("    - {}", dir.display());
        }
    }

    println!("\n{}", "Search Results:".bold());
    if files.is_empty() {
        println!("  No files found matching the criteria.");
    } else {
        println!("  Found {} file(s):", files.len());
        for file in files {
            println!("  - {}", file);
        }
    }

    if !permission_denied_dirs.is_empty() {
        eprintln!("\n{}", "Permission Denied:".red().bold());
        for dir in permission_denied_dirs {
            eprintln!("  - {}", dir.red());
        }
    }

    if other_error_occurred {
        eprintln!("\n{}", "Errors:".red().bold());
        for error in error_messages.lines() {
            eprintln!("  {}", error.red());
        }
    }

    println!("\n{}", "Search completed.".bold());
}

