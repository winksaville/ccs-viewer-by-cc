# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

See [Foramt details](README.md#todo-format)

 - Replace serde_json::Value fields with typed structs or custom Opaque newtype [17]
 - Add error test data and library tests for known-bad inputs [18]
 - Improve error output formatting (columnize, full paths) [19]
 - Refactor common metadata fields into a shared SessionMetadata struct [4]
 - Have claude code design claude-code a session viewer [1]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

 - Fix remaining deserialization errors across ~/data/prgs (7371 → 0 errors, 0.12.0) [15]
 - Add first-line sniff test, exit codes, CLI flag cleanup (0.12.0) [16]
 - Define serde structs for JSONL deserialization (0.1.0) [2]
 - Add queue-operation, system, custom-title, agent-name record types (0.2.0) [3]
 - Support text blocks in user content arrays (0.3.0) [5]
 - Make SystemRecord subtype-specific fields optional (0.4.0) [6]
 - Add clap CLI with -V, multi-file args, and all-variants-covered test (0.5.0) [7]
 - Add all-optional-fields-seen test, deny_unknown_fields, fix missing fields (0.6.0) [8]
 - Add all-optional-fields-seen test, reorder struct fields, fix sourceToolAssistantUUID rename (0.6.0) [8]
 - Compact single-line output with grouped error summary (0.7.0) [9]
 - Add unknown fields from vc-x1 sessions (0.8.0) [10]
 - Add CLI flags: list, errors, recursive, glob (0.9.0) [11]
 - Add agent meta.json support (0.10.0) [13]
 - Add agentId to record structs for agent JSONL (0.11.0) [14]

 - test 3 lb signs [20]

# References

[1]: chores-01.md#have-claude-code-design-a-claude-code-session-viewer
[2]: chores-01.md#define-serde-structs-for-jsonl-deserialization
[3]: chores-01.md#add-new-record-type-variants
[4]: chores-01.md#refactor-sessionmetadata
[5]: chores-01.md#support-text-blocks-in-user-content-arrays
[6]: chores-01.md#make-systemrecord-subtype-specific-fields-optional
[7]: chores-01.md#add-clap-cli
[8]: chores-01.md#add-all-optional-fields-seen-test
[9]: chores-01.md#compact-single-line-output-with-grouped-errors
[10]: chores-01.md#add-unknown-fields-from-vc-x1-sessions
[11]: chores-01.md#add-cli-flags-list-errors-recursive-glob-20260324-090
[12]: chores-01.md#add-support-for-agent-session-files
[13]: chores-01.md#add-agent-metajson-support-20260324-0100
[14]: chores-01.md#add-agentid-to-record-structs-for-agent-jsonl-20260324-0110
[15]: chores-01.md#fix-remaining-deserialization-errors-0120
[16]: chores-01.md#sniff-test-and-exit-code-cleanup-0120-dev11
[17]: chores-01.md#replace-serdejsonvalue-with-typed-structs
[18]: chores-01.md#add-error-test-data-and-library-tests
[19]: chores-01.md#improve-error-output-formatting
[20]: chores-01.md#test-3-lb-signs