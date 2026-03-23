use serde::Deserialize;
use serde_json::Value;

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
}

// ---------------------------------------------------------------------------
// file-history-snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileHistorySnapshotRecord {
    pub message_id: String,
    pub snapshot: Snapshot,
    pub is_snapshot_update: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub message_id: String,
    pub tracked_file_backups: Value,
    pub timestamp: String,
}

// ---------------------------------------------------------------------------
// user
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRecord {
    pub parent_uuid: Option<String>,
    pub is_sidechain: bool,
    pub prompt_id: Option<String>,
    pub message: UserMessage,
    pub uuid: String,
    pub timestamp: String,
    // Present only on initial user prompt
    pub permission_mode: Option<String>,
    // Present on tool result records
    pub tool_use_result: Option<Value>,
    pub source_tool_assistant_uuid: Option<String>,
    // Common metadata
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
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
    Structured(Value),
}

// ---------------------------------------------------------------------------
// assistant
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantRecord {
    pub parent_uuid: String,
    pub is_sidechain: bool,
    pub message: AssistantMessage,
    pub request_id: Option<String>,
    pub uuid: String,
    pub timestamp: String,
    // Common metadata
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssistantMessage {
    pub model: String,
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub role: String,
    pub content: Vec<AssistantContentBlock>,
    pub stop_reason: Option<String>,
    pub stop_sequence: Option<String>,
    pub usage: Usage,
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
        input: Value,
        caller: Option<Value>,
    },
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: Option<u64>,
    pub cache_read_input_tokens: Option<u64>,
    pub cache_creation: Option<CacheCreation>,
    pub service_tier: Option<String>,
    pub inference_geo: Option<String>,
    pub server_tool_use: Option<Value>,
    pub iterations: Option<Value>,
    pub speed: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CacheCreation {
    #[serde(rename = "ephemeral_5m_input_tokens")]
    pub ephemeral_5m_input_tokens: u64,
    #[serde(rename = "ephemeral_1h_input_tokens")]
    pub ephemeral_1h_input_tokens: u64,
}

// ---------------------------------------------------------------------------
// progress
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressRecord {
    pub parent_uuid: String,
    pub is_sidechain: bool,
    pub data: ProgressData,
    #[serde(rename = "parentToolUseID")]
    pub parent_tool_use_id: Option<String>,
    #[serde(rename = "toolUseID")]
    pub tool_use_id: Option<String>,
    pub timestamp: String,
    pub uuid: String,
    // Common metadata
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressData {
    #[serde(rename = "type")]
    pub data_type: String,
    pub hook_event: Option<String>,
    pub hook_name: Option<String>,
    pub command: Option<String>,
}

// ---------------------------------------------------------------------------
// last-prompt
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastPromptRecord {
    pub last_prompt: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// queue-operation
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueOperationRecord {
    pub operation: String,
    pub timestamp: String,
    pub session_id: String,
    pub content: Option<String>,
}

// ---------------------------------------------------------------------------
// system
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemRecord {
    pub parent_uuid: String,
    pub is_sidechain: bool,
    pub subtype: String,
    pub timestamp: String,
    pub uuid: String,
    pub is_meta: bool,
    // subtype: turn_duration
    pub duration_ms: Option<u64>,
    // subtype: local_command
    pub content: Option<String>,
    pub level: Option<String>,
    // common metadata
    pub user_type: Option<String>,
    pub entrypoint: Option<String>,
    pub cwd: Option<String>,
    pub session_id: Option<String>,
    pub version: Option<String>,
    pub git_branch: Option<String>,
    pub slug: Option<String>,
}

// ---------------------------------------------------------------------------
// custom-title
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomTitleRecord {
    pub custom_title: String,
    pub session_id: String,
}

// ---------------------------------------------------------------------------
// agent-name
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentNameRecord {
    pub agent_name: String,
    pub session_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    fn deserialize_file(path: &str) -> usize {
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
            223
        );
    }

    #[test]
    fn deserialize_sample_86fb() {
        assert_eq!(
            deserialize_file("data/86fb7a89-abfa-4e84-b862-5983e93c0b3b.jsonl"),
            301
        );
    }
}
