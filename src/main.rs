use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

use ccs_viewer::types::Record;

#[derive(Parser)]
#[command(version, about = "Claude Code session JSONL viewer")]
struct Cli {
    /// File or directory glob patterns to process
    #[arg(required = true)]
    patterns: Vec<String>,

    /// Show per-file summary lines
    #[arg(short, long)]
    list: bool,

    /// Show grouped error details
    #[arg(short, long)]
    errors: bool,

    /// Recursively search directories for matching files
    #[arg(short, long)]
    recursive: bool,

    /// File glob pattern for recursive mode (repeatable, default: *.jsonl)
    #[arg(long = "glob")]
    globs: Vec<String>,
}

/// Resolve CLI patterns into a list of file paths.
fn resolve_files(cli: &Cli) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if cli.recursive {
        let file_globs: Vec<&str> = if cli.globs.is_empty() {
            vec!["*.jsonl"]
        } else {
            cli.globs.iter().map(|s| s.as_str()).collect()
        };

        // Expand positional patterns as directory globs.
        for pattern in &cli.patterns {
            let dirs: Vec<PathBuf> = match glob::glob(pattern) {
                Ok(paths) => paths
                    .filter_map(|p| p.ok())
                    .filter(|p| p.is_dir())
                    .collect(),
                Err(e) => {
                    eprintln!("Bad pattern {pattern:?}: {e}");
                    std::process::exit(1);
                }
            };
            if dirs.is_empty() {
                eprintln!("No directories match {pattern:?}");
                std::process::exit(1);
            }
            for dir in &dirs {
                for fg in &file_globs {
                    let full = format!("{}/**/{fg}", dir.display());
                    match glob::glob(&full) {
                        Ok(paths) => {
                            for entry in paths.flatten() {
                                if entry.is_file() {
                                    files.push(entry);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Bad glob {full:?}: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
    } else {
        let file_globs: Vec<&str> = if cli.globs.is_empty() {
            vec!["*.jsonl"]
        } else {
            cli.globs.iter().map(|s| s.as_str()).collect()
        };

        for pattern in &cli.patterns {
            // Check if pattern is a literal directory path first.
            let path = PathBuf::from(pattern);
            if path.is_dir() {
                // Directory: expand file globs inside it (non-recursive).
                for fg in &file_globs {
                    let full = format!("{}/{fg}", path.display());
                    match glob::glob(&full) {
                        Ok(paths) => {
                            for entry in paths.flatten() {
                                if entry.is_file() {
                                    files.push(entry);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Bad glob {full:?}: {e}");
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                // Treat as file glob pattern.
                match glob::glob(pattern) {
                    Ok(paths) => {
                        let mut matched = Vec::new();
                        for entry in paths.flatten() {
                            if entry.is_file() {
                                matched.push(entry);
                            } else if entry.is_dir() {
                                // Glob matched a directory: expand file
                                // globs inside it (non-recursive).
                                for fg in &file_globs {
                                    let full = format!("{}/{fg}", entry.display());
                                    if let Ok(inner) = glob::glob(&full) {
                                        for f in inner.flatten() {
                                            if f.is_file() {
                                                matched.push(f);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if matched.is_empty() {
                            eprintln!("No files match {pattern:?}");
                            std::process::exit(1);
                        }
                        files.extend(matched);
                    }
                    Err(e) => {
                        eprintln!("Bad pattern {pattern:?}: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }

    files.sort();
    files.dedup();
    files
}

/// Key for grouping errors: the serde error message + record type.
#[derive(Hash, Eq, PartialEq)]
struct ErrorKey {
    message: String,
    record_type: String,
}

/// Tracks one group of identical errors across files.
struct ErrorGroup {
    count: usize,
    /// One representative file:line for grabbing test data.
    example_file: String,
    example_line: usize,
    /// How many distinct files contain this error.
    file_count: usize,
    /// Track which files we've seen (by index) to count distinct files.
    seen_files: Vec<usize>,
}

fn main() {
    let cli = Cli::parse();
    let files = resolve_files(&cli);

    if files.is_empty() {
        eprintln!("No files to process");
        std::process::exit(1);
    }

    let mut total_records: usize = 0;
    let mut total_errors: usize = 0;
    let mut error_groups: HashMap<ErrorKey, ErrorGroup> = HashMap::new();

    for (file_idx, path) in files.iter().enumerate() {
        let file = File::open(path).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {e}", path.display());
            std::process::exit(1);
        });

        let reader = BufReader::new(file);
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        let mut file_errors: usize = 0;
        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        for (i, line) in reader.lines().enumerate() {
            let line = line.unwrap_or_else(|e| {
                eprintln!("Error reading line {}: {e}", i + 1);
                std::process::exit(1);
            });
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Record>(&line) {
                Ok(record) => {
                    *counts.entry(record.label()).or_insert(0) += 1;
                }
                Err(e) => {
                    file_errors += 1;
                    let record_type = serde_json::from_str::<serde_json::Value>(&line)
                        .ok()
                        .and_then(|v| v.get("type")?.as_str().map(String::from))
                        .unwrap_or_else(|| "unknown".to_string());

                    let key = ErrorKey {
                        message: format!("{e}"),
                        record_type,
                    };
                    let group = error_groups.entry(key).or_insert_with(|| ErrorGroup {
                        count: 0,
                        example_file: filename.clone(),
                        example_line: i + 1,
                        file_count: 0,
                        seen_files: Vec::new(),
                    });
                    group.count += 1;
                    if !group.seen_files.contains(&file_idx) {
                        group.seen_files.push(file_idx);
                        group.file_count += 1;
                    }
                }
            }
        }

        let file_total: usize = counts.values().sum();
        total_records += file_total;
        total_errors += file_errors;

        if cli.list {
            let mut parts = vec![
                format!("errors: {file_errors}"),
                format!("records: {file_total}"),
            ];
            for (label, count) in &counts {
                parts.push(format!("{label}: {count}"));
            }
            println!("{filename}: {}", parts.join(", "));
        }
    }

    let file_count = files.len();
    println!(
        "{}Summary: {file_count} file(s), {total_records} records, {total_errors} errors",
        if cli.list { "\n" } else { "" }
    );

    if cli.errors && !error_groups.is_empty() {
        println!("\nErrors:");
        let mut groups: Vec<_> = error_groups.into_iter().collect();
        groups.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        for (key, group) in &groups {
            println!(
                "  {}x {} in {} ({}:{} in {} file(s))",
                group.count,
                key.message,
                key.record_type,
                group.example_file,
                group.example_line,
                group.file_count,
            );
        }
    }

    if total_errors > 0 {
        std::process::exit(1);
    }
}
