use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use ccs_viewer::types::Record;

fn main() {
    let path = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: ccs-viewer <session.jsonl>");
        std::process::exit(1);
    });

    let file = File::open(&path).unwrap_or_else(|e| {
        eprintln!("Error opening {path}: {e}");
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
                let label = match &record {
                    Record::FileHistorySnapshot(_) => "file-history-snapshot",
                    Record::User(_) => "user",
                    Record::Assistant(_) => "assistant",
                    Record::Progress(_) => "progress",
                    Record::LastPrompt(_) => "last-prompt",
                };
                *counts.entry(label).or_insert(0) += 1;
            }
            Err(e) => {
                errors.push((i + 1, format!("{e}")));
            }
        }
    }

    let total: usize = counts.values().sum();
    println!("Deserialized {total} records from {path}");
    for (label, count) in &counts {
        println!("  {label}: {count}");
    }
    if !errors.is_empty() {
        eprintln!("\n{} error(s):", errors.len());
        for (line_num, msg) in &errors {
            eprintln!("  line {line_num}: {msg}");
        }
        std::process::exit(1);
    }
}
