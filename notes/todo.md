# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

See [Foramt details](README.md#todo-format)

 - Add clap CLI with -V, multi-file args, and all-variants-covered test [7]
 - Refactor common metadata fields into a shared SessionMetadata struct [4]
 - Have claude code design claude-code a session viewer [1]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

 - Define serde structs for JSONL deserialization (0.1.0) [2]
 - Add queue-operation, system, custom-title, agent-name record types (0.2.0) [3]
 - Support text blocks in user content arrays (0.3.0) [5]
 - Make SystemRecord subtype-specific fields optional (0.4.0) [6]



# References

[1]: chores-01.md#have-claude-code-design-a-claude-code-session-viewer
[2]: chores-01.md#define-serde-structs-for-jsonl-deserialization
[3]: chores-01.md#add-new-record-type-variants
[4]: chores-01.md#refactor-sessionmetadata
[5]: chores-01.md#support-text-blocks-in-user-content-arrays
[6]: chores-01.md#make-systemrecord-subtype-specific-fields-optional
[7]: chores-01.md#add-clap-cli
