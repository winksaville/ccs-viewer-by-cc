//! Serde types for Claude Code session JSONL files.
//!
//! Each line of a session file is a JSON object with a `"type"` field that
//! determines its shape. The [`Record`] enum uses serde's internally tagged
//! representation (`#[serde(tag = "type")]`) to dispatch to the correct
//! variant struct.
//!
//! # Key patterns
//!
//! **Label strings** — The `RECORD_LABELS` const is the single source of
//! truth for type name strings (e.g. `"assistant"`, `"system"`).
//! [`Record::label()`] indexes into it; [`Record::all_labels()`] returns
//! the full slice. This avoids duplicating strings between the two methods.
//!
//! **Struct field ordering** — Each struct lists required fields first,
//! then `Option` fields below a separator comment. This makes it visually
//! obvious which fields need entries in the `optional_fields()` list.
//!
//! **`deny_unknown_fields`** — Every struct has `#[serde(deny_unknown_fields)]`
//! so that any JSON key not mapped to a Rust field causes a deserialization
//! error. This ensures we know immediately when Claude Code adds new fields.
//!
//! **`optional_fields()`** — Structs with `Option` fields have a companion
//! `optional_fields()` method listing the camelCase (or snake_case) JSON
//! names of those fields. The `all_optional_fields_seen` test uses these
//! lists to verify every `Option` field is `Some` at least once in test
//! data. Nested fields use dot notation (`"message.usage.speed"`) and
//! array filtering uses brackets (`"message.content[tool_use].caller"`).
//!
//! **Warning**: The `optional_fields()` lists are static — adding an
//! `Option` field to a struct without updating the list means that field
//! silently goes untested. The separator comment in each struct serves as
//! a reminder.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

/// Wrapper for JSON data we haven't given a typed struct yet.
/// Signals "structured but not yet typed" — distinguishes from
/// deliberately modeled fields. Grep for `Untyped` to find all
/// remaining untyped fields.
#[derive(Debug, Deserialize)]
pub struct Untyped(#[allow(dead_code)] Value);

/// Top-level record: each line of a Claude Code session JSONL file
/// deserializes into one of these variants, discriminated by the `type` field.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Record {
    #[serde(rename = "file-history-snapshot")]
    FileHistorySnapshot(FileHistorySnapshotRecord),

    #[serde(rename = "user")]
    User(Box<UserRecord>),

    #[serde(rename = "assistant")]
    Assistant(Box<AssistantRecord>),

    #[serde(rename = "progress")]
    Progress(Box<ProgressRecord>),

    #[serde(rename = "last-prompt")]
    LastPrompt(LastPromptRecord),

    #[serde(rename = "queue-operation")]
    QueueOperation(QueueOperationRecord),

    #[serde(rename = "system")]
    System(Box<SystemRecord>),

    #[serde(rename = "custom-title")]
    CustomTitle(CustomTitleRecord),

    #[serde(rename = "agent-name")]
    AgentName(AgentNameRecord),

    #[serde(rename = "summary")]
    Summary(SummaryRecord),
}

/// Single source of truth for record type label strings.
const RECORD_LABELS: &[&str] = &[
    "file-history-snapshot",
    "user",
    "assistant",
    "progress",
    "last-prompt",
    "queue-operation",
    "system",
    "custom-title",
    "agent-name",
    "summary",
];

impl Record {
    /// Returns the type label string for this record variant.
    pub fn label(&self) -> &'static str {
        RECORD_LABELS[match self {
            Record::FileHistorySnapshot(_) => 0,
            Record::User(_) => 1,
            Record::Assistant(_) => 2,
            Record::Progress(_) => 3,
            Record::LastPrompt(_) => 4,
            Record::QueueOperation(_) => 5,
            Record::System(_) => 6,
            Record::CustomTitle(_) => 7,
            Record::AgentName(_) => 8,
            Record::Summary(_) => 9,
        }]
    }

    /// Returns the full list of known record type labels.
    pub fn all_labels() -> &'static [&'static str] {
        RECORD_LABELS
    }

    /// Returns the optional fields list for each record type that has them.
    /// Used by tests to verify all Option fields are exercised.
    pub fn optional_fields_by_type() -> Vec<(&'static str, &'static [&'static str])> {
        vec![
            ("user", UserRecord::optional_fields()),
            ("assistant", AssistantRecord::optional_fields()),
            ("progress", ProgressRecord::optional_fields()),
            ("queue-operation", QueueOperationRecord::optional_fields()),
            ("system", SystemRecord::optional_fields()),
        ]
    }
}

// ---------------------------------------------------------------------------
// file-history-snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileHistorySnapshotRecord {
    pub message_id: String,
    pub snapshot: Snapshot,
    pub is_snapshot_update: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Snapshot {
    pub message_id: String,
    pub tracked_file_backups: HashMap<String, FileBackupEntry>,
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileBackupEntry {
    pub backup_file_name: Option<String>,
    pub version: u64,
    pub backup_time: String,
}

// ---------------------------------------------------------------------------
// user
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UserRecord {
    pub is_sidechain: bool,
    pub message: UserMessage,
    pub uuid: String,
    pub timestamp: String,
    // --- Option fields (add camelCase JSON name to optional_fields below) ---
    pub is_meta: Option<bool>,
    // --- Option fields (add camelCase JSON name to optional_fields below) ---
    pub agent_id: Option<String>,
    pub parent_uuid: Option<String>,
    pub prompt_id: Option<String>,
    pub permission_mode: Option<String>,
    pub tool_use_result: Option<Untyped>,
    #[serde(rename = "sourceToolAssistantUUID")]
    pub source_tool_assistant_uuid: Option<String>,
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
    pub plan_content: Option<String>,
    pub todos: Option<Untyped>,
    pub thinking_metadata: Option<ThinkingMetadata>,
    pub is_visible_in_transcript_only: Option<bool>,
    pub is_compact_summary: Option<bool>,
}

// WARNING: When adding an Option field above, add its camelCase JSON name here.
// This list cannot detect missing entries — unlisted fields silently go untested.
impl UserRecord {
    pub fn optional_fields() -> &'static [&'static str] {
        &[
            "agentId",
            "parentUuid",
            "promptId",
            "permissionMode",
            "toolUseResult",
            "isMeta",
            "sourceToolAssistantUUID",
            "userType",
            "entrypoint",
            "cwd",
            "sessionId",
            "version",
            "gitBranch",
            "slug",
            "planContent",
            "todos",
            "thinkingMetadata",
            "isVisibleInTranscriptOnly",
            "isCompactSummary",
            // UserContentBlock::ToolResult
            "message.content[tool_result].is_error",
        ]
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserMessage {
    pub role: String,
    pub content: UserContent,
}

/// User message content is either a plain text string (initial prompt)
/// or an array of content blocks (tool results, text, etc.).
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum UserContent {
    Text(String),
    Blocks(Vec<UserContentBlock>),
}

/// A block within a user message content array.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum UserContentBlock {
    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
        is_error: Option<bool>,
    },

    #[serde(rename = "text")]
    Text { text: String },
}

/// Tool result content can be a plain string or a structured value.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Structured(Untyped),
}

// ---------------------------------------------------------------------------
// assistant
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AssistantRecord {
    pub is_sidechain: bool,
    pub message: AssistantMessage,
    pub uuid: String,
    pub timestamp: String,
    // --- Option fields (add camelCase JSON name to optional_fields below) ---
    pub parent_uuid: Option<String>,
    pub agent_id: Option<String>,
    pub is_api_error_message: Option<bool>,
    pub request_id: Option<String>,
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
    pub error: Option<String>,
}

// WARNING: When adding an Option field above (or in nested structs below),
// add its camelCase JSON name here. Nested fields use dot notation.
// This list cannot detect missing entries — unlisted fields silently go untested.
impl AssistantRecord {
    pub fn optional_fields() -> &'static [&'static str] {
        &[
            "parentUuid",
            "agentId",
            "isApiErrorMessage",
            "requestId",
            "userType",
            "entrypoint",
            "cwd",
            "sessionId",
            "version",
            "gitBranch",
            "slug",
            "error",
            // AssistantMessage (snake_case — no rename_all on this struct)
            "message.stop_reason",
            "message.stop_sequence",
            // Usage (snake_case — no rename_all on this struct)
            "message.usage.cache_creation_input_tokens",
            "message.usage.cache_read_input_tokens",
            "message.usage.cache_creation",
            "message.usage.service_tier",
            "message.usage.inference_geo",
            "message.usage.server_tool_use",
            "message.usage.iterations",
            "message.usage.speed",
            // "message.container" — always null in practice
            // "message.context_management" — always null in practice
            // AssistantContentBlock::ToolUse
            "message.content[tool_use].caller",
        ]
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssistantMessage {
    pub model: String,
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<AssistantContentBlock>,
    pub usage: Usage,
    // --- Option fields (listed in AssistantRecord::optional_fields) ---
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub container: Option<Untyped>,
    pub context_management: Option<Untyped>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum AssistantContentBlock {
    #[serde(rename = "thinking")]
    Thinking { thinking: String, signature: String },

    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: Untyped,
        // --- Option field (listed in AssistantRecord::optional_fields) ---
        caller: Option<Caller>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    // --- Option fields (listed in AssistantRecord::optional_fields) ---
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation: Option<CacheCreation>,
    pub service_tier: Option<String>,
    pub inference_geo: Option<String>,
    pub server_tool_use: Option<ServerToolUse>,
    pub iterations: Option<Untyped>,
    pub speed: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CacheCreation {
    #[serde(rename = "ephemeral_5m_input_tokens")]
    pub ephemeral_5m_input_tokens: u64,
    #[serde(rename = "ephemeral_1h_input_tokens")]
    pub ephemeral_1h_input_tokens: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerToolUse {
    pub web_search_requests: u64,
    pub web_fetch_requests: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Caller {
    #[serde(rename = "type")]
    pub caller_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ThinkingMetadata {
    // --- Option fields (all optional — two distinct shapes in the wild) ---
    pub level: Option<String>,
    pub disabled: Option<bool>,
    pub triggers: Option<Vec<Untyped>>,
    pub max_thinking_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CompactMetadata {
    // --- Option fields (two distinct shapes in the wild) ---
    pub removed_messages: Option<u64>,
    pub trigger: Option<String>,
    pub pre_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiError {
    pub status: u16,
    pub headers: Untyped,
    #[serde(rename = "requestID")]
    pub request_id: Option<String>,
}

// ---------------------------------------------------------------------------
// progress
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProgressRecord {
    pub parent_uuid: String,
    pub is_sidechain: bool,
    pub data: ProgressData,
    pub timestamp: String,
    pub uuid: String,
    // --- Option fields (add camelCase JSON name to optional_fields below) ---
    pub agent_id: Option<String>,
    #[serde(rename = "parentToolUseID")]
    pub parent_tool_use_id: Option<String>,
    #[serde(rename = "toolUseID")]
    pub tool_use_id: Option<String>,
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
}

// WARNING: When adding an Option field above (or in ProgressData below),
// add its camelCase JSON name here. Nested fields use dot notation.
// This list cannot detect missing entries — unlisted fields silently go untested.
impl ProgressRecord {
    pub fn optional_fields() -> &'static [&'static str] {
        &[
            "agentId",
            "parentToolUseID",
            "toolUseID",
            "userType",
            "entrypoint",
            "cwd",
            "sessionId",
            "version",
            "gitBranch",
            "slug",
            // ProgressData
            "data.hookEvent",
            "data.hookName",
            "data.command",
            "data.message",
            "data.prompt",
            "data.agentId",
            "data.query",
            "data.resultCount",
            "data.output",
            "data.fullOutput",
            "data.elapsedTimeSeconds",
            "data.taskId",
            "data.timeoutMs",
            "data.totalBytes",
            "data.totalLines",
            "data.normalizedMessages",
            "data.taskDescription",
            "data.taskType",
        ]
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ProgressData {
    #[serde(rename = "type")]
    pub data_type: String,
    // --- Option fields (listed in ProgressRecord::optional_fields) ---
    pub hook_event: Option<String>,
    pub hook_name: Option<String>,
    pub command: Option<String>,
    pub message: Option<Untyped>,
    pub prompt: Option<String>,
    pub agent_id: Option<String>,
    pub query: Option<String>,
    pub result_count: Option<u64>,
    pub output: Option<String>,
    pub full_output: Option<String>,
    pub elapsed_time_seconds: Option<u64>,
    pub task_id: Option<String>,
    pub timeout_ms: Option<u64>,
    pub total_bytes: Option<u64>,
    pub total_lines: Option<u64>,
    pub normalized_messages: Option<Untyped>,
    pub task_description: Option<String>,
    pub task_type: Option<String>,
}

// ---------------------------------------------------------------------------
// last-prompt
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LastPromptRecord {
    pub last_prompt: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// queue-operation
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueueOperationRecord {
    pub operation: String,
    pub timestamp: String,
    pub session_id: String,
    // --- Option fields (add camelCase JSON name to optional_fields below) ---
    pub content: Option<Untyped>,
}

// WARNING: When adding an Option field above, add its camelCase JSON name here.
// This list cannot detect missing entries — unlisted fields silently go untested.
impl QueueOperationRecord {
    pub fn optional_fields() -> &'static [&'static str] {
        &["content"]
    }
}

// ---------------------------------------------------------------------------
// system
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SystemRecord {
    pub is_sidechain: bool,
    pub subtype: String,
    pub timestamp: String,
    pub uuid: String,
    // --- Option fields (add camelCase JSON name to optional_fields below) ---
    pub parent_uuid: Option<String>,
    pub is_meta: Option<bool>,
    pub agent_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub content: Option<String>,
    pub level: Option<String>,
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
    pub error: Option<ApiError>,
    pub logical_parent_uuid: Option<String>,
    pub compact_metadata: Option<CompactMetadata>,
    pub message_count: Option<u64>,
    pub retry_in_ms: Option<f64>,
    pub retry_attempt: Option<u64>,
    pub max_retries: Option<u64>,
}

// WARNING: When adding an Option field above, add its camelCase JSON name here.
// This list cannot detect missing entries — unlisted fields silently go untested.
impl SystemRecord {
    pub fn optional_fields() -> &'static [&'static str] {
        &[
            "parentUuid",
            "isMeta",
            "agentId",
            "durationMs",
            "content",
            "level",
            "userType",
            "entrypoint",
            "cwd",
            "sessionId",
            "version",
            "gitBranch",
            "slug",
            "error",
            "logicalParentUuid",
            "compactMetadata",
            "messageCount",
            "retryInMs",
            "retryAttempt",
            "maxRetries",
        ]
    }
}

// ---------------------------------------------------------------------------
// custom-title
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CustomTitleRecord {
    pub custom_title: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// agent-name
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AgentNameRecord {
    pub agent_name: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// summary
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SummaryRecord {
    pub summary: String,
    pub leaf_uuid: String,
}

// ---------------------------------------------------------------------------
// agent meta (agent-*.meta.json — standalone JSON, not JSONL records)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AgentMeta {
    pub agent_type: String,
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    const TEST_FILES: &[&str] = &[
        "data/31ba272b-f7c2-436b-a017-269e27a64d07.jsonl",
        "data/997afb98-c6aa-4f4a-92ac-a841e040414b.jsonl",
        "data/092de687-cd0d-4583-b872-bc2908dff3ba.jsonl",
        "data/86fb7a89-abfa-4e84-b862-5983e93c0b3b.jsonl",
        "data/ccs-viewer-tests.jsonl",
        "data/agent-test.jsonl",
    ];

    fn deserialize_file(path: &str) -> usize {
        deserialize_file_collecting(path, None)
    }

    fn deserialize_file_collecting(path: &str, mut labels: Option<&mut HashSet<String>>) -> usize {
        let file = File::open(path).expect("sample JSONL file should exist");
        let reader = BufReader::new(file);
        let mut count = 0;
        for (i, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            if line.trim().is_empty() {
                continue;
            }
            let result = serde_json::from_str::<Record>(&line);
            assert!(
                result.is_ok(),
                "line {} failed: {}",
                i + 1,
                result.unwrap_err()
            );
            if let (Ok(record), Some(labels)) = (&result, labels.as_deref_mut()) {
                labels.insert(record.label().to_string());
            }
            count += 1;
        }
        count
    }

    #[test]
    fn deserialize_sample_31ba() {
        assert_eq!(
            deserialize_file("data/31ba272b-f7c2-436b-a017-269e27a64d07.jsonl"),
            22
        );
    }

    #[test]
    fn deserialize_sample_997a() {
        assert_eq!(
            deserialize_file("data/997afb98-c6aa-4f4a-92ac-a841e040414b.jsonl"),
            461
        );
    }

    #[test]
    fn deserialize_sample_092d() {
        assert_eq!(
            deserialize_file("data/092de687-cd0d-4583-b872-bc2908dff3ba.jsonl"),
            224
        );
    }

    #[test]
    fn deserialize_sample_86fb() {
        assert_eq!(
            deserialize_file("data/86fb7a89-abfa-4e84-b862-5983e93c0b3b.jsonl"),
            301
        );
    }

    /// Look up a dotted path in a serde_json::Value.
    /// Supports segments like "message", "usage", or array filtering
    /// like "content[tool_use]" (find element in array where type == "tool_use").
    fn value_at_path<'a>(val: &'a Value, path: &str) -> Option<&'a Value> {
        let mut current = val;
        for segment in path.split('.') {
            if let Some(idx) = segment.find('[') {
                let key = &segment[..idx];
                let filter_type = &segment[idx + 1..segment.len() - 1];
                let arr = current.get(key)?.as_array()?;
                current = arr
                    .iter()
                    .find(|el| el.get("type").and_then(|t| t.as_str()) == Some(filter_type))?;
            } else {
                current = current.get(segment)?;
            }
        }
        if current.is_null() {
            None
        } else {
            Some(current)
        }
    }

    /// Verify every Option field in every struct has at least one non-null
    /// occurrence across test data. This catches Option fields that are defined
    /// in a struct but never actually exercised by any test file.
    ///
    /// WARNING: This test uses a static list of Option fields per record type
    /// (see each struct's `optional_fields()` method). It cannot catch Option
    /// fields that are missing from those lists — those fields will silently
    /// remain untested (always None). When adding a new Option field to a
    /// struct, you must also add its camelCase JSON name to the corresponding
    /// `optional_fields()` list.
    #[test]
    fn all_optional_fields_seen() {
        let field_map = Record::optional_fields_by_type();
        let mut seen: HashSet<(String, String)> = HashSet::new();

        for path in TEST_FILES {
            let file = File::open(path).expect("test file should exist");
            let reader = BufReader::new(file);
            for line in reader.lines() {
                let line = line.unwrap();
                if line.trim().is_empty() {
                    continue;
                }
                let val: Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                let record_type = match val.get("type").and_then(|t| t.as_str()) {
                    Some(t) => t,
                    None => continue,
                };
                for &(rtype, fields) in &field_map {
                    if rtype != record_type {
                        continue;
                    }
                    for field in fields {
                        if value_at_path(&val, field).is_some() {
                            seen.insert((record_type.to_string(), field.to_string()));
                        }
                    }
                }
            }
        }

        let mut never_seen = Vec::new();
        for &(rtype, fields) in &field_map {
            for field in fields {
                if !seen.contains(&(rtype.to_string(), field.to_string())) {
                    never_seen.push(format!("{rtype}.{field}"));
                }
            }
        }
        never_seen.sort();
        assert!(
            never_seen.is_empty(),
            "Option fields never seen as Some in test data: {never_seen:?}"
        );
    }

    #[test]
    fn deserialize_agent_jsonl() {
        assert_eq!(deserialize_file("data/agent-test.jsonl"), 7);
    }

    #[test]
    fn deserialize_agent_meta() {
        let path = "data/agent-adf742daa2c66fe48.meta.json";
        let file = File::open(path).expect("agent meta file should exist");
        let meta: AgentMeta = serde_json::from_reader(file).expect("agent meta should deserialize");
        assert_eq!(meta.agent_type, "Explore");
        assert!(meta.description.is_some());
    }

    #[test]
    fn deserialize_agent_meta_no_description() {
        let path = "data/agent-ada8e90bba5770efc.meta.json";
        let file = File::open(path).expect("agent meta file should exist");
        let meta: AgentMeta = serde_json::from_reader(file).expect("agent meta should deserialize");
        assert_eq!(meta.agent_type, "Explore");
        assert!(meta.description.is_none());
    }

    #[test]
    fn all_variants_covered() {
        let mut seen = HashSet::new();
        for path in TEST_FILES {
            deserialize_file_collecting(path, Some(&mut seen));
        }
        let expected: HashSet<String> =
            Record::all_labels().iter().map(|s| s.to_string()).collect();
        let missing: Vec<_> = expected.difference(&seen).collect();
        assert!(
            missing.is_empty(),
            "Record variants not covered by test data: {missing:?}"
        );
    }

    // -----------------------------------------------------------------------
    // Error tests — verify known-bad inputs are rejected
    // -----------------------------------------------------------------------

    const ERR_DATA_DIR: &str = "err-data";

    fn assert_record_err(filename: &str) {
        let path = format!("{ERR_DATA_DIR}/{filename}");
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        for (i, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            assert!(
                serde_json::from_str::<Record>(line).is_err(),
                "{path}:{} should fail to deserialize but succeeded",
                i + 1,
            );
        }
    }

    fn assert_meta_err(filename: &str) {
        let path = format!("{ERR_DATA_DIR}/{filename}");
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        assert!(
            serde_json::from_str::<AgentMeta>(&content).is_err(),
            "{path} should fail to deserialize but succeeded",
        );
    }

    #[test]
    fn err_unknown_variant() {
        assert_record_err("unknown-variant.jsonl");
    }

    #[test]
    fn err_unknown_field() {
        assert_record_err("unknown-field.jsonl");
    }

    #[test]
    fn err_wrong_type() {
        assert_record_err("wrong-type.jsonl");
    }

    #[test]
    fn err_missing_field() {
        assert_record_err("missing-field.jsonl");
    }

    #[test]
    fn err_bad_json() {
        assert_record_err("bad-json.jsonl");
    }

    #[test]
    fn err_bad_deser() {
        assert_record_err("bad-deser.jsonl");
    }

    #[test]
    fn err_bad_meta() {
        assert_meta_err("bad-meta.meta.json");
    }
}
