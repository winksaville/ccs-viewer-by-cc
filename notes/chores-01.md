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

## Add unknown fields from vc-x1 sessions (20260324 0.8.0)

  Running `ccs-viewer` against `../vc-x1/.claude/*.jsonl` (25 files,
  ~11.8k records) surfaced 7 unknown fields across 345 errors.
  Working the list bottom-up (least frequent first):

  - [x] `planContent` (String) in user — 1 occurrence, 1 file
  - [x] `todos` (Value) in user — 3 occurrences, 1 file
  - [x] `error` (String) in assistant — 5 occurrences, 3 files
  - [x] `query` (String) in progress.data — 7 occurrences, 3 files
  - [x] `resultCount` (u64) in progress.data — 7 occurrences, 3 files
  - [x] `output` (String) in progress.data — 78 occurrences, 2 files
        Also found on same test line: `fullOutput` (String),
        `elapsedTimeSeconds` (u64), `taskId` (String), `timeoutMs` (u64),
        `totalBytes` (u64), `totalLines` (u64)
  - [x] `normalizedMessages` (Value) in progress.data — 244 occurrences, 2 files

  Test data for new fields goes in `data/ccs-viewer-tests.jsonl`.

  ### Process for each field

  1. Run `ccs-viewer` to get the error list with file:line references.
  2. Inspect the field's type and value from the reported file:line:
     ```
     sed -n '<line>p' <file> | python3 -c \
       "import sys,json; d=json.load(sys.stdin); \
        v=d.get('<field>') or d.get('data',{}).get('<field>'); \
        print(type(v).__name__, json.dumps(v)[:200])"
     ```
  3. Append that line to `data/ccs-viewer-tests.jsonl`:
     ```
     sed -n '<line>p' <file> >> data/ccs-viewer-tests.jsonl
     ```
  4. Add `Option<T>` field to the struct in `src/types.rs`
     (under the `--- Option fields ---` separator).
  5. Add the camelCase JSON name to the struct's `optional_fields()`.
  6. Run: `cargo fmt && cargo clippy && cargo test`
  7. Install: `cargo install --path .`
  8. Verify:
     - `ccs-viewer data/ccs-viewer-tests.jsonl` (new field works)
     - `ccs-viewer data/*.jsonl` (regression — 0 errors on local data)
  9. Update the checklist in this chore section.

## Add CLI flags: list, errors, recursive, glob (20260324 0.9.0)

  Rework CLI to default to summary-only output with opt-in detail.
  Add `glob` crate for program-side glob expansion (portable, no shell
  dependency).

  ### Design

  Two modes based on `-r`:

  **Without `-r`** — positional args are file glob patterns expanded
  by the program:
  ```
  ccs-viewer "data/*.jsonl"
  ccs-viewer "../*/.claude/*.jsonl"
  ```

  **With `-r`** — positional args are directories (or directory globs)
  searched recursively. `--glob` filters which files match (default:
  `*.jsonl`). Multiple `--glob` flags allowed:
  ```
  ccs-viewer -r .claude ../vc-x1/.claude
  ccs-viewer -r --glob "*.jsonl" --glob "*.json" .claude
  ```

  ### Flags

  - `-l` / `--list` — show per-file summary lines (default: off)
  - `-e` / `--errors` — show grouped error detail section (default: off)
  - `-r` / `--recursive` — recursive directory search
  - `--glob <PATTERN>` — file pattern for recursive mode (repeatable,
    default: `*.jsonl`)
  - Summary line always shown

  ### Implementation

  1. Add `glob` crate dependency
  2. Rework `Cli` struct: positional args become glob patterns,
     add `-l`, `-e`, `-r`, `--glob` flags
  3. Build file list: without `-r`, expand positional globs as file
     paths; with `-r`, expand positional globs as directories then
     walk them matching `--glob` patterns
  4. Process files as before, but only print per-file lines if `-l`
  5. Only print error section if `-e`
  6. Always print summary
  7. Directory positional args (without `-r`): auto-expand `--glob`
     patterns inside the directory (non-recursive)

## Add support for agent session files

  Recursive search (`-r`) picks up `agent-*.jsonl` files alongside
  main session files. These have a different record format and
  currently cause ~10k deserialization errors across 28 files in
  vc-x1. Need to add record types for the agent session format.

## Add agent meta.json support (20260324 0.10.0)

  Added `AgentMeta` struct to parse `agent-*.meta.json` files — small
  standalone JSON files with `agentType` and `description` fields.

  Changes:
  - `types.rs`: new `AgentMeta` struct with `deny_unknown_fields`
  - `main.rs`: detect `.meta.json` files by filename suffix and parse
    as single-object JSON instead of line-by-line JSONL. Counted as
    "agent-meta" in the summary output.
  - Default `--glob` patterns now include `*.meta.json` alongside
    `*.jsonl`, so recursive mode picks them up automatically.
  - Added `deserialize_agent_meta` test with real test data copied
    from `.claude/` subagents directory.

  This is the first step toward full agent session support. The
  `agent-*.jsonl` files still need `agentId` fields added to the
  record structs (next task).

## Add agentId to record structs for agent JSONL (20260324 0.11.0)

  Agent subagent JSONL files (`agent-*.jsonl`) have the same record
  format as main session files but include an `agentId` field on
  every record. Added `agent_id: Option<String>` to all four record
  structs that appear in agent sessions:

  - `UserRecord`
  - `AssistantRecord`
  - `ProgressRecord`
  - `SystemRecord`

  Added `agentId` to each struct's `optional_fields()` list and
  created `data/agent-test.jsonl` with 7 records covering all four
  types.

  Also made `AgentMeta.description` optional — vc-x1 has meta.json
  files with only `agentType` and no `description`. Added test data
  from vc-x1 for the no-description case.

  Verified: 0 errors across 66 files in vc-x1/.claude and all local
  test data.

## Fix remaining deserialization errors across all sessions (20260325 0.12.0)

  Running `ccs-viewer . -e -r` from `~/data/prgs` hits 5077 files,
  137k records, and 7371 errors (after removing 3 benchmark logs.jsonl
  files that accounted for ~60M false positives).

  ### Error inventory (12 categories, 7371 total)

  **Simple field additions** (add as Option to existing structs):
  1. 2963x `thinkingMetadata` unknown in user (309 files)
  2. 26x `isVisibleInTranscriptOnly` unknown in user (21 files)
  3. 2106x `data_mtime` unknown in agent-meta (2106 files)
  4. 9x `id` unknown in agent-meta (9 files)
  5. 2x `taskDescription` unknown in progress (1 file)
  6. 1x `error` unknown in system (1 file)

  **Type fixes** (field type mismatch):
  7. 1475x null string in assistant (1475 files) — a required String
     field is sometimes null; need sample data to identify which
  8. 49x null string in system (43 files) — same issue
  9. 42x sequence instead of string in queue-operation (21 files) —
     `content` is sometimes an array; change Option<String> to Option<Value>

  **New variant:**
  10. 689x unknown variant `summary` (250 files) — add Summary variant

  **Unfixable / out of scope:**
  11. 5x missing field `type` in line2.jsonl (2 files) — not CCS files
  12. 4x malformed JSON at column 1 (1 file) — corrupt data

  ### Plan

  - dev0: version bump + this chores section
  - dev1: simple field additions (#1–#6, ~5107 errors)
  - dev1.1: first-line sniff test to skip non-CCS .jsonl files (#11)
  - dev2: type fixes (#7–#9, ~1566 errors)
  - dev3: new summary variant (#10, 689 errors)
  - 0.12.0: final release, remove -devN

  ### dev1: simple field additions

  Added optional fields to existing structs:
  - `UserRecord`: `thinkingMetadata` (Value), `isVisibleInTranscriptOnly`
    (bool), `isCompactSummary` (bool — discovered alongside
    `isVisibleInTranscriptOnly`)
  - `ProgressData`: `taskDescription` (String), `taskType` (String —
    discovered alongside `taskDescription`)
  - `SystemRecord`: `error` (Value), `retryInMs` (f64), `retryAttempt`
    (u64), `maxRetries` (u64 — discovered in same sample record)

  Also made `SystemRecord.isMeta` optional — one record (the api_error
  with `error` field) lacked it entirely.

  Fixed false positives:
  - `AgentMeta` errors (#3, #4) were mypy cache `*.meta.json` files,
    not agent meta files. Narrowed default glob from `*.meta.json` to
    `agent-*.meta.json` — eliminates 2115 false positives.

  ### dev1.1: sniff test + exit code cleanup

  First-line sniff test: skip .jsonl files whose first line doesn't
  start with `{"type":` or `{"parentUuid":`. This eliminates non-CCS
  files (benchmark logs, line2.jsonl, etc.) without relying on filename
  patterns. Added `--skipped` / `-s` flag to list skipped files.

  Exit code rework:
  - 0: success (default, even with deserialization errors)
  - 1: tool failure (bad args, can't open file, no files match)
  - 2: deserialization errors present (only with `--strict`)

  Summary line now shows skipped count when non-zero.

  ### dev2: type fixes

  Fixed type mismatches where fields had the wrong type:

  - `AssistantRecord.parentUuid`: String → Option<String>
    (null in agent sessions without a parent)
  - `SystemRecord.parentUuid`: String → Option<String>
    (null in agent sessions without a parent)
  - `QueueOperationRecord.content`: Option<String> →
    Option<Value> (sometimes an array of content blocks)

  Also discovered and fixed two more unknown system fields
  (both from context pruning records, 26x in 21 files):
  - `logicalParentUuid`: Option<String>
  - `compactMetadata`: Option<Value>

  ### Progress

  ```
  Before (0.11.0):  5077 files, 137041 records,  7371 errors (12 categories)
  dev1 (0.12.0):    2965 files, 140374 records,  2264 errors (6 categories)
  dev1.1 (0.12.0):  2687 files, 140451 records,  2259 errors (5 categories), 278 skipped
  dev2 (0.12.0):    2689 files, 142333 records,   693 errors (2 categories), 278 skipped
  ```

  Remaining errors (for dev3):
  - 689x unknown variant `summary` (250 files)
  - 4x malformed JSON (unfixable)

  ### dev2.1: add -E flag for error file paths

  Added `-E`/`--error-files` flag: like `-e` but also lists all
  file paths (with line numbers) for each error group. Reworked
  `ErrorGroup` to store all file:line hits instead of just one
  example. The error summary line now shows the full path of the
  first hit instead of just the filename.

  `-E` implies `-e` behavior — no need to pass both.

## Replace serde_json::Value with typed structs

  Several struct fields use `serde_json::Value` as a catch-all for
  JSON we haven't fully typed yet. This works but loses type safety
  and doesn't catch schema changes via `deny_unknown_fields`.

  Fields currently using Value:
  - `UserRecord`: thinkingMetadata, toolUseResult, todos
  - `SystemRecord`: error, compactMetadata
  - `ProgressData`: message, prompt, normalizedMessages,
    serverToolUse, iterations
  - `QueueOperationRecord`: content
  - `AssistantRecord` nested: serverToolUse, iterations,
    caller, container, contextManagement

  Options:
  1. Typed structs per field — best safety, most work, risks
     breakage on upstream schema changes
  2. Custom `Opaque(Value)` newtype — signals "structured but
     not yet typed", distinguishes from deliberately modeled
     fields, can incrementally promote to real structs

  If we go with a catch-all, prefer our own type over bare
  `serde_json::Value` so grep can find all untyped fields and
  we can add validation or logging in one place later.
