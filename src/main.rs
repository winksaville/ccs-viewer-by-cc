use std::collections::HashMap;
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

fn main() {
    let cli = Cli::parse();
    let mut total_records: usize = 0;
    let mut total_errors: usize = 0;

    for path in &cli.files {
        let file = File::open(path).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {e}", path.display());
            std::process::exit(1);
        });

        let reader = BufReader::new(file);
        let mut counts: HashMap<&str, usize> = HashMap::new();
        let mut errors: Vec<(usize, String)> = Vec::new();

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
                    errors.push((i + 1, format!("{e}")));
                }
            }
        }

        let file_total: usize = counts.values().sum();
        total_records += file_total;
        total_errors += errors.len();
        println!("Deserialized {file_total} records from {}", path.display());
        for (label, count) in &counts {
            println!("  {label}: {count}");
        }
        if !errors.is_empty() {
            eprintln!("\n{} error(s):", errors.len());
            for (line_num, msg) in &errors {
                eprintln!("  line {line_num}: {msg}");
            }
        }
    }

    let file_count = cli.files.len();
    if total_errors > 0 {
        eprintln!("\nSummary: {file_count} files, {total_records} records, {total_errors} errors");
        std::process::exit(1);
    } else {
        println!("\nSummary: {file_count} files, {total_records} records, 0 errors");
    }
}
