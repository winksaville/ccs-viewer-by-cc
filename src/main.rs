use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use clap::Parser;

use ccs_viewer::types::{AgentMeta, Record};

#[derive(Parser)]
#[command(
    version,
    about = "Claude Code session JSONL viewer",
    after_help = "\
Output order: per-file list (-l) > errors (-e/-E) > skipped (-s) > empty (-z) > summary (always last).

Summary: <total> total files, <valid> valid files with <records> records[, <n> empty][, <n> skipped][, <n> errors]
  total:   all files found by glob/recursive search
  valid:   files successfully processed (total minus empty and skipped)
  records: total successfully deserialized records (in valid files)
  empty:   zero-length files (shown when non-zero)
  skipped: files that failed the first-line sniff test (shown when non-zero)
  errors:  total deserialization failures (shown when non-zero)

Exit codes:
  0  Success (default, even with deserialization errors)
  1  Tool failure (bad args, can't open file, no files match)
  2  Deserialization errors present (only with --strict)"
)]
struct Cli {
    /// File or directory glob patterns to process
    #[arg(required = true)]
    patterns: Vec<String>,

    /// Show per-file summary lines (one line per file with record type counts)
    #[arg(short, long)]
    list: bool,

    /// Show grouped error details (deduplicated by message, sorted by count)
    #[arg(short, long)]
    errors: bool,

    /// Show error details with full file paths for each error group
    #[arg(short = 'E', long)]
    error_files: bool,

    /// Recursively search directories for matching files
    #[arg(short, long)]
    recursive: bool,

    /// File glob pattern for recursive mode (repeatable, default: *.jsonl, agent-*.meta.json)
    #[arg(long = "glob")]
    globs: Vec<String>,

    /// Exit 2 if deserialization errors are present
    #[arg(long)]
    strict: bool,

    /// Show files skipped by the first-line sniff test
    #[arg(short, long)]
    skipped: bool,

    /// Show empty (zero-length) files
    #[arg(short, long)]
    zero: bool,
}

/// Resolve CLI patterns into a list of file paths.
fn resolve_files(cli: &Cli) -> Vec<PathBuf> {
    let mut files = Vec::new();

    if cli.recursive {
        let file_globs: Vec<&str> = if cli.globs.is_empty() {
            vec!["*.jsonl", "agent-*.meta.json"]
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
            vec!["*.jsonl", "agent-*.meta.json"]
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

/// A single file:line occurrence of an error.
struct ErrorHit {
    path: String,
    line: usize,
}

/// Tracks one group of identical errors across files.
struct ErrorGroup {
    count: usize,
    /// How many distinct files contain this error.
    file_count: usize,
    /// Track which files we've seen (by index) to count distinct files.
    seen_files: Vec<usize>,
    /// All file:line occurrences (one per distinct file).
    hits: Vec<ErrorHit>,
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
    let mut total_skipped: usize = 0;
    let mut total_empty: usize = 0;
    let mut skipped_files: Vec<String> = Vec::new();
    let mut empty_files: Vec<String> = Vec::new();
    let mut error_groups: HashMap<ErrorKey, ErrorGroup> = HashMap::new();

    for (file_idx, path) in files.iter().enumerate() {
        let file = File::open(path).unwrap_or_else(|e| {
            eprintln!("Error opening {}: {e}", path.display());
            std::process::exit(1);
        });

        let filename = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        // Handle .meta.json files as single-object JSON (not JSONL).
        if filename.ends_with(".meta.json") {
            let result: Result<AgentMeta, _> = serde_json::from_reader(file);
            match result {
                Ok(_meta) => {
                    total_records += 1;
                    if cli.list {
                        println!("{filename}: agent-meta (ok)");
                    }
                }
                Err(e) => {
                    total_errors += 1;
                    let key = ErrorKey {
                        message: format!("{e}"),
                        record_type: "agent-meta".to_string(),
                    };
                    let group = error_groups.entry(key).or_insert_with(|| ErrorGroup {
                        count: 0,
                        file_count: 0,
                        seen_files: Vec::new(),
                        hits: Vec::new(),
                    });
                    group.count += 1;
                    if !group.seen_files.contains(&file_idx) {
                        group.seen_files.push(file_idx);
                        group.file_count += 1;
                        group.hits.push(ErrorHit {
                            path: path.display().to_string(),
                            line: 1,
                        });
                    }
                    if cli.list {
                        println!("{filename}: agent-meta (error)");
                    }
                }
            }
            continue;
        }

        let mut reader = BufReader::new(file);

        // Check for empty files before the sniff test.
        let mut first_line = String::new();
        if reader.read_line(&mut first_line).unwrap_or(0) == 0 {
            total_empty += 1;
            if cli.zero {
                empty_files.push(path.display().to_string());
            }
            continue;
        }

        // First-line sniff test: skip .jsonl files that don't look like
        // Claude Code sessions. CCS files start with {"type": or {"parentUuid":.
        let trimmed = first_line.trim();
        if !trimmed.starts_with("{\"type\":") && !trimmed.starts_with("{\"parentUuid\":") {
            total_skipped += 1;
            if cli.skipped {
                skipped_files.push(path.display().to_string());
            }
            continue;
        }

        // Process lines starting from the first line we already read.
        let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
        let mut file_errors: usize = 0;

        // Process first line, then remaining lines.
        let remaining_lines = reader.lines().map(|l| l.unwrap_or_default());
        let all_lines = std::iter::once(first_line.trim_end().to_string()).chain(remaining_lines);

        for (i, line) in all_lines.enumerate() {
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
                        file_count: 0,
                        seen_files: Vec::new(),
                        hits: Vec::new(),
                    });
                    group.count += 1;
                    if !group.seen_files.contains(&file_idx) {
                        group.seen_files.push(file_idx);
                        group.file_count += 1;
                        group.hits.push(ErrorHit {
                            path: path.display().to_string(),
                            line: i + 1,
                        });
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
    let processed = file_count - total_skipped - total_empty;

    let show_errors = cli.errors || cli.error_files;
    if show_errors && !error_groups.is_empty() {
        println!("{}Errors:", if cli.list { "\n" } else { "" });
        let mut groups: Vec<_> = error_groups.into_iter().collect();
        groups.sort_by(|a, b| b.1.count.cmp(&a.1.count));
        for (key, group) in &groups {
            let first = &group.hits[0];
            println!(
                "  {}x {} in {} ({}:{} in {} file(s))",
                group.count, key.message, key.record_type, first.path, first.line, group.file_count,
            );
            if cli.error_files {
                for hit in &group.hits {
                    println!("    {}:{}", hit.path, hit.line);
                }
            }
        }
        println!();
    }

    if cli.skipped && !skipped_files.is_empty() {
        println!("Skipped:");
        for f in &skipped_files {
            println!("  {f}");
        }
        println!();
    }

    if cli.zero && !empty_files.is_empty() {
        println!("Empty:");
        for f in &empty_files {
            println!("  {f}");
        }
        println!();
    }

    let total_files = processed + total_skipped + total_empty;
    let mut suffix = String::new();
    if total_empty > 0 {
        suffix.push_str(&format!(", {total_empty} empty"));
    }
    if total_skipped > 0 {
        suffix.push_str(&format!(", {total_skipped} skipped"));
    }
    if total_errors > 0 {
        suffix.push_str(&format!(", {total_errors} errors"));
    }
    println!(
        "Summary: {total_files} total files, {processed} valid files with {total_records} records{suffix}"
    );

    if cli.strict && total_errors > 0 {
        std::process::exit(2);
    }
}
