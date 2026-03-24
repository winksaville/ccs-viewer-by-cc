# Chores-01

Discussions and notes on various chores in github compatible markdown.
There is also a [todo.md](todo.md) file and it tracks tasks and in
general there should be a chore section for each task with the why
and how this task will be completed.

See [Chores format](README.md#chores-format)

## Have claude code design a claude-code session viewer

  In `hwr.claude/` is a .jsonl file with a set of json lines representing me asking
  claude-code to "reqaquaint" (sp: reaquaint) itself with a trivial app created
  using, `cargo new hwr` and `hwr.claude/` is the directory `~/.claude/projects/<path>/`.

  Medium term want to convert the jsonl lines into a the "conversation".
  In the short term I'd like to create the rust structs that each represent
  the information, I don't want to process it I just want to be able to
  use serde to deserialize the data in the file into the set of rust structs.
  
  I think each line should destruct into a single struct, but I suspect
  this "super" struct is composed of several substructures and those
  should be defined separately and used but the "super" struct.
  
  In the end we'll have an app that can read a full conversation with
  claude-code and display it. Long term this capability will be combined
  with the app repo and we'll create a coherent story of why, how and what
  for the app.

  Take a look at the file and let me know what you think

## Define serde structs for JSONL deserialization (20260323 0.1.0)

  Created `src/types.rs` with serde structs to deserialize all 5 JSONL record
  types from Claude Code session files:

  - `Record` — top-level enum, internally tagged on `"type"` field
  - `FileHistorySnapshotRecord` / `Snapshot` — file backup snapshots
  - `UserRecord` / `UserMessage` / `UserContent` — user prompts + tool results
  - `AssistantRecord` / `AssistantMessage` / `AssistantContentBlock` — model responses
  - `ProgressRecord` / `ProgressData` — hook execution events
  - `LastPromptRecord` — session end marker
  - `Usage` / `CacheCreation` — API usage metadata

  Key design decisions:
  - Internally tagged enum (`#[serde(tag = "type")]`) for `Record` and content blocks
  - `#[serde(untagged)]` for `UserContent` (string vs tool_result array)
  - `serde_json::Value` for polymorphic fields (`toolUseResult`, `tool_use.input`)
  - `Box<T>` on large enum variants per clippy `large_enum_variant`
  - camelCase rename on Claude Code wrapper structs; snake_case (default) on API structs

  Verified: all 22 lines of sample JSONL deserialize successfully.

## Add new record type variants (20260323 0.2.0)

  Added 4 new variants to the `Record` enum discovered in the larger session
  file `data/997afb98-...jsonl` (461 records):

  - `QueueOperation` — slash command queue events (`enqueue`/`remove`)
  - `System` — turn duration metadata (subtype `turn_duration`)
  - `CustomTitle` — user-set session title
  - `AgentName` — agent name assignment

  Also updated the test to cover both data files (moved path from
  `hwr.claude/` to `data/`) and added the new variant labels to `main.rs`.

## Refactor SessionMetadata

  Extract the 7 common metadata fields (`user_type`, `entrypoint`, `cwd`,
  `session_id`, `version`, `git_branch`, `slug`) repeated across
  `UserRecord`, `AssistantRecord`, `ProgressRecord`, and `SystemRecord`
  into a shared `SessionMetadata` struct using `#[serde(flatten)]`.
  Make individual fields required `String` instead of `Option<String>` —
  they appear on every record in the data we have.

## Support text blocks in user content arrays (20260323 0.3.0)

  User message content arrays can contain both `tool_result` and `text` blocks.
  Replaced `Vec<ToolResultBlock>` with `Vec<UserContentBlock>` where
  `UserContentBlock` is a tagged enum (`#[serde(tag = "type")]`) handling both
  block types. Removed the standalone `ToolResultBlock` struct.

  Discovered in session file `data/092de687-...jsonl` (line 81) where a user
  follow-up text was interleaved with tool results in the same content array.

## Make SystemRecord subtype-specific fields optional (20260323 0.4.0)

  `SystemRecord` has multiple subtypes (`turn_duration`, `local_command`) with
  different fields. Made `duration_ms` optional (only on `turn_duration`) and
  added optional `content` and `level` fields (only on `local_command`).

  Discovered in `data/86fb7a89-...jsonl` (2222 records) where `local_command`
  system records lacked `durationMs`.

## Add clap CLI (20260323 0.5.0)

  Switched from manual arg parsing to `clap` (derive). Gets `-V`/`--version`
  for free from Cargo.toml, proper `--help`, and multi-file argument support
  (`ccs-viewer data/*.jsonl`).

  Added `Record::label()` method and `Record::all_labels()` to centralize
  variant label strings. Added `all_variants_covered` test that asserts every
  `Record` variant appears at least once across all test data files.

## Add all-optional-fields-seen test (20260323 0.6.0)

  Added `all_optional_fields_seen` test that verifies every `Option` field
  in every struct is `Some` at least once across test data. Each struct with
  Option fields has an `optional_fields()` method listing camelCase JSON names
  right next to the struct definition. Nested fields use dot notation
  (e.g. `"message.usage.speed"`), array filtering uses bracket notation
  (e.g. `"message.content[tool_use].caller"`).

  Reordered struct fields: required fields first, Option fields grouped at
  the bottom with a separator comment.

  Found and fixed a real bug: `sourceToolAssistantUUID` was never being
  deserialized because `rename_all = "camelCase"` produced
  `sourceToolAssistantUuid` but the JSON key uses all-caps `UUID`.
  Added explicit `#[serde(rename = "sourceToolAssistantUUID")]`.

  Added `deny_unknown_fields` to all structs so serde rejects any JSON key
  not mapped to a struct field. This surfaced several missing fields:
  - `UserRecord.is_meta` (Option<bool>)
  - `AssistantRecord.is_api_error_message` (Option<bool>)
  - `AssistantMessage.container` (Option<Value>) — always null
  - `AssistantMessage.context_management` (Option<Value>) — always null
  - `ProgressData.message` (Option<Value>)
  - `ProgressData.prompt` (Option<Value>)
  - `ProgressData.agent_id` (Option<String>)

  Grabbed a line from `.claude/` with `stop_sequence` set and appended it to
  the `092de687` test file so that field is now tested too.

  `container` and `context_management` are excluded from the optional_fields
  test list — always null in practice.

## Compact single-line output with grouped errors (20260324 0.7.0)

  Reformatted CLI output so each file produces a single summary line:

  ```
  filename.jsonl: errors: 0, records: 220, assistant: 65, user: 50, ...
  ```

  Record types are alphabetically sorted (BTreeMap). After all files,
  a summary line shows totals. If there are errors, they're grouped by
  serde error message + record type and printed as:

  ```
  3x unknown field "foo" in assistant (filename.jsonl:142 in 2 files)
  ```

  Each error line includes: occurrence count, full serde message, record
  type (peeked from raw JSON), one file:line example for grabbing test
  data, and the number of files affected.

  No more need to go to the file to identify the record type — it's
  right in the error line.

