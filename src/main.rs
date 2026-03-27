use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use clap::Parser;

use ccs_viewer::types::{AgentMeta, AssistantContentBlock, Record, UserContent, UserContentBlock};

#[derive(Parser)]
#[command(
    version,
    about = "Claude Code session JSONL viewer",
    after_help = "\
Output is always in this fixed order regardless of flag order:
  valid (-v) > errors (-e/-E) > skipped (-s) > zero-len (-z) > summary (always last)

Summary: <total> total files, <valid> valid files with <records> records, <n> zero-len, <n> skipped, <n> errors
  total:    all files found by glob/recursive search
  valid:    files successfully processed (total minus zero-len and skipped)
  records:  total successfully deserialized records (in valid files)
  zero-len: zero-length files
  skipped:  files that failed the first-line sniff test
  errors:   total deserialization failures

Exit codes:
  0  Success (default, even with deserialization errors)
  1  Tool failure (bad args, can't open file, no files match)
  2  Deserialization errors present (only with --strict)"
)]
struct Cli {
    /// File or directory glob patterns to process
    #[arg(required = true)]
    patterns: Vec<String>,

    /// Recursively search directories for matching files
    #[arg(short, long)]
    recursive: bool,

    /// File glob pattern for recursive mode (repeatable, default: *.jsonl, agent-*.meta.json)
    #[arg(long = "glob")]
    globs: Vec<String>,

    /// Show valid files (filename + record type counts on two lines)
    #[arg(short, long, help_heading = "Summary detail flags")]
    valid: bool,

    /// Show error file paths with line numbers
    #[arg(short, long, help_heading = "Summary detail flags")]
    errors: bool,

    /// Show deduplicated error summary (grouped by message, sorted by count)
    #[arg(short = 'E', long, help_heading = "Summary detail flags")]
    error_summary: bool,

    /// Show files skipped by the first-line sniff test
    #[arg(short, long, help_heading = "Summary detail flags")]
    skipped: bool,

    /// Show zero-length files
    #[arg(short, long, help_heading = "Summary detail flags")]
    zero: bool,

    /// Exit 2 if deserialization errors are present
    #[arg(long)]
    strict: bool,

    /// Display session transcript (user/assistant conversation)
    #[arg(long)]
    show: bool,
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

/// Stats returned by `show_transcript`.
#[derive(Debug, PartialEq)]
struct ShowStats {
    shown: usize,
    thinking_only: usize,
    thinking_empty: usize,
    skipped: usize,
    parse_errors: usize,
}

/// Render a session transcript to `out`, returning stats.
fn show_transcript_to(reader: impl BufRead, out: &mut impl std::io::Write) -> ShowStats {
    let mut stats = ShowStats {
        shown: 0,
        thinking_only: 0,
        thinking_empty: 0,
        skipped: 0,
        parse_errors: 0,
    };

    for line in reader.lines() {
        let line = line.unwrap_or_default();
        if line.trim().is_empty() {
            continue;
        }
        let record: Record = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => {
                stats.parse_errors += 1;
                continue;
            }
        };
        match record {
            Record::User(user_rec) => {
                if stats.shown > 0 {
                    writeln!(out, "---------").unwrap();
                }
                writeln!(out, "--- user ---").unwrap();
                match user_rec.message.content {
                    UserContent::Text(text) => writeln!(out, "{text}").unwrap(),
                    UserContent::Blocks(blocks) => {
                        for block in blocks {
                            match block {
                                UserContentBlock::Text { text } => {
                                    writeln!(out, "{text}").unwrap();
                                }
                                UserContentBlock::ToolResult { .. } => {
                                    writeln!(out, "[tool_result]").unwrap();
                                }
                            }
                        }
                    }
                }
                stats.shown += 1;
            }
            Record::Assistant(asst_rec) => {
                // Collect visible content; skip records with only thinking blocks.
                let mut lines = Vec::new();
                let mut has_thinking = false;
                let mut has_empty_thinking = false;
                for block in &asst_rec.message.content {
                    match block {
                        AssistantContentBlock::Text { text } => lines.push(text.clone()),
                        AssistantContentBlock::ToolUse { name, .. } => {
                            lines.push(format!("[tool: {name}]"));
                        }
                        AssistantContentBlock::Thinking { thinking, .. } => {
                            has_thinking = true;
                            if thinking.is_empty() {
                                has_empty_thinking = true;
                            }
                        }
                    }
                }
                if has_thinking && has_empty_thinking {
                    stats.thinking_empty += 1;
                }
                if lines.is_empty() {
                    stats.thinking_only += 1;
                    continue;
                }
                if stats.shown > 0 {
                    writeln!(out, "---------").unwrap();
                }
                writeln!(out, "--- assistant ---").unwrap();
                for line in &lines {
                    writeln!(out, "{line}").unwrap();
                }
                stats.shown += 1;
            }
            _ => {
                stats.skipped += 1;
            }
        }
    }

    stats
}

/// Display a session transcript to stdout.
fn show_transcript(path: &PathBuf) {
    let file = File::open(path).unwrap_or_else(|e| {
        eprintln!("Error opening {}: {e}", path.display());
        std::process::exit(1);
    });
    let reader = BufReader::new(file);
    let mut out = std::io::stdout().lock();
    let stats = show_transcript_to(reader, &mut out);

    writeln!(out).unwrap();
    writeln!(
        out,
        "Show summary: {} shown, {} thinking-only ({} empty), {} skipped, {} errors",
        stats.shown, stats.thinking_only, stats.thinking_empty, stats.skipped, stats.parse_errors
    )
    .unwrap();
}

fn main() {
    let cli = Cli::parse();
    let files = resolve_files(&cli);

    if files.is_empty() {
        eprintln!("No files to process");
        std::process::exit(1);
    }

    if cli.show {
        if files.len() > 1 {
            eprintln!("--show requires a single file, got {}", files.len());
            std::process::exit(1);
        }
        show_transcript(&files[0]);
        return;
    }

    let mut total_records: usize = 0;
    let mut total_errors: usize = 0;
    let mut total_skipped: usize = 0;
    let mut total_empty: usize = 0;
    let mut skipped_files: Vec<String> = Vec::new();
    let mut empty_files: Vec<String> = Vec::new();
    let mut error_groups: HashMap<ErrorKey, ErrorGroup> = HashMap::new();
    let mut valid_header_printed = false;

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
            let metadata = file.metadata().ok();
            if metadata.as_ref().is_some_and(|m| m.len() == 0) {
                total_empty += 1;
                if cli.zero {
                    empty_files.push(path.display().to_string());
                }
                continue;
            }
            let result: Result<AgentMeta, _> = serde_json::from_reader(file);
            match result {
                Ok(_meta) => {
                    total_records += 1;
                    if cli.valid {
                        if !valid_header_printed {
                            println!("Valid:");
                            valid_header_printed = true;
                        }
                        println!("  {}", path.display());
                        println!("    agent-meta (ok)");
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
                    if cli.valid {
                        if !valid_header_printed {
                            println!("Valid:");
                            valid_header_printed = true;
                        }
                        println!("  {}", path.display());
                        println!("    agent-meta (error)");
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

        if cli.valid {
            if !valid_header_printed {
                println!("Valid:");
                valid_header_printed = true;
            }
            let mut parts = vec![
                format!("errors: {file_errors}"),
                format!("records: {file_total}"),
            ];
            for (label, count) in &counts {
                parts.push(format!("{label}: {count}"));
            }
            println!("  {}", path.display());
            println!("    {}", parts.join(", "));
        }
    }

    let file_count = files.len();
    let processed = file_count - total_skipped - total_empty;

    if valid_header_printed {
        println!();
    }

    let show_errors = cli.errors || cli.error_summary;
    if show_errors && !error_groups.is_empty() {
        let mut groups: Vec<_> = error_groups.iter().collect();
        groups.sort_by(|a, b| b.1.count.cmp(&a.1.count));

        if cli.error_summary && cli.errors {
            // -e -E combined: grouped summary with all paths nested
            println!("Error summary:");
            for (key, group) in &groups {
                println!("  {}x {} in {}", group.count, key.message, key.record_type,);
                for hit in &group.hits {
                    println!("    {}:{}", hit.path, hit.line);
                }
            }
            println!();
        } else if cli.error_summary {
            // -E only: grouped summary, two lines per group
            println!("Error summary:");
            for (key, group) in &groups {
                let first = &group.hits[0];
                println!("  {}x {} in {}", group.count, key.message, key.record_type,);
                println!(
                    "    {} file(s), first: {}:{}",
                    group.file_count, first.path, first.line,
                );
            }
            println!();
        } else if cli.errors {
            // -e only: flat list of all error file:line paths
            println!("Errors:");
            let mut all_hits: Vec<_> = error_groups.values().flat_map(|g| g.hits.iter()).collect();
            all_hits.sort_by(|a, b| a.path.cmp(&b.path).then(a.line.cmp(&b.line)));
            for hit in all_hits {
                println!("  {}:{}", hit.path, hit.line);
            }
            println!();
        }
    }

    if cli.skipped && !skipped_files.is_empty() {
        println!("Skipped:");
        for f in &skipped_files {
            println!("  {f}");
        }
        println!();
    }

    if cli.zero && !empty_files.is_empty() {
        println!("Zero-len:");
        for f in &empty_files {
            println!("  {f}");
        }
        println!();
    }

    let total_files = processed + total_skipped + total_empty;
    println!(
        "Summary: {total_files} total files, {processed} valid files with {total_records} records, {total_empty} zero-len, {total_skipped} skipped, {total_errors} errors"
    );

    if cli.strict && total_errors > 0 {
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn show(path: &str) -> (String, ShowStats) {
        let file = File::open(path).expect("test file should exist");
        let reader = BufReader::new(file);
        let mut buf = Vec::new();
        let stats = show_transcript_to(reader, &mut buf);
        (String::from_utf8(buf).unwrap(), stats)
    }

    #[test]
    fn show_basic_conversation() {
        let (out, stats) = show("data/31ba272b-f7c2-436b-a017-269e27a64d07.jsonl");
        assert_eq!(stats.shown, 15);
        assert_eq!(stats.thinking_only, 1);
        assert_eq!(stats.thinking_empty, 1);
        assert_eq!(stats.skipped, 6);
        assert_eq!(stats.parse_errors, 0);
        assert!(out.contains("--- user ---\nreqaquaint"));
        assert!(out.contains("[tool: Bash]"));
        assert!(out.contains("[tool_result]"));
    }

    #[test]
    fn show_nonempty_thinking() {
        let (_, stats) = show("data/ccs-viewer-tests.jsonl");
        assert_eq!(stats.thinking_only, 1);
        assert_eq!(stats.thinking_empty, 0);
    }

    #[test]
    fn show_mixed_errors() {
        let (out, stats) = show("err-data/show-mixed.jsonl");
        assert_eq!(stats.shown, 3);
        assert_eq!(stats.parse_errors, 2);
        assert_eq!(stats.skipped, 0);
        assert!(out.contains("--- user ---\nhello\n"));
        assert!(out.contains("--- assistant ---\nHi there!\n"));
        assert!(out.contains("--- user ---\ngoodbye\n"));
    }

    #[test]
    fn show_all_errors() {
        let (out, stats) = show("err-data/bad-json.jsonl");
        assert_eq!(stats.shown, 0);
        assert_eq!(stats.parse_errors, 1);
        assert!(out.is_empty());
    }

    #[test]
    fn show_empty_input() {
        let reader = Cursor::new(b"");
        let mut buf = Vec::new();
        let stats = show_transcript_to(reader, &mut buf);
        assert_eq!(stats.shown, 0);
        assert_eq!(stats.parse_errors, 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn show_separator_between_records() {
        let (out, stats) = show("err-data/show-mixed.jsonl");
        assert_eq!(stats.shown, 3);
        // Separator appears between records but not before the first
        assert!(out.starts_with("--- user ---\n"));
        assert_eq!(out.matches("---------").count(), 2);
    }
}
