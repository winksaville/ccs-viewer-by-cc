use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

use ccs_viewer::types::Record;

#[derive(Parser)]
#[command(version, about = "Claude Code session JSONL viewer")]
struct Cli {
    /// Session JSONL files to process
    #[arg(required = true)]
    files: Vec<PathBuf>,
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
    let mut total_records: usize = 0;
    let mut total_errors: usize = 0;
    let mut error_groups: HashMap<ErrorKey, ErrorGroup> = HashMap::new();

    for (file_idx, path) in cli.files.iter().enumerate() {
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
                    // Peek at "type" field for error context.
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

        // Single summary line per file.
        let mut parts = vec![
            format!("errors: {file_errors}"),
            format!("records: {file_total}"),
        ];
        for (label, count) in &counts {
            parts.push(format!("{label}: {count}"));
        }
        println!("{filename}: {}", parts.join(", "));
    }

    // Summary line.
    let file_count = cli.files.len();
    println!("\nSummary: {file_count} file(s), {total_records} records, {total_errors} errors");

    // Error detail section.
    if !error_groups.is_empty() {
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
        std::process::exit(1);
    }
}
