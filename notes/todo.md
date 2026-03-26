# Todo

This file contains near term tasks with a short description
and reference links to more details.

## In Progress

## Todo

A markdown list of task to do in the near feature

See [Foramt details](README.md#todo-format)

 - Refactor common metadata fields into a shared SessionMetadata struct [4]
 - Have claude code design claude-code a session viewer [1]

## Done

Completed tasks are moved from `## Todo` to here, `## Done`, as they are completed
and older `## Done` sections are moved to [done.md](done.md) to keep this file small.

 - Define serde structs for JSONL deserialization (0.1.0) [2]
 - Add queue-operation, system, custom-title, agent-name record types (0.2.0) [3]
 - Support text blocks in user content arrays (0.3.0) [5]
 - Make SystemRecord subtype-specific fields optional (0.4.0) [6]
 - Add clap CLI with -V, multi-file args, and all-variants-covered test (0.5.0) [7]
 - Add all-optional-fields-seen test (0.6.0) [8]
 - Compact single line output with grouped errors 0.7.0 [9]
 - Add unknown fields from vc x1 sessions 0.8.0 [10]
 - Add cli flags list errors recursive glob 0.9.0 [11]
 - Add support for agent session files [12]
 - Add agent metajson support 0.10.0 [13]
 - Add agentid to record structs for agent jsonl 0.11.0 [14]
 - Add new optional fields 0.12.0-dev1 [15]
 - Sniff test and exit code cleanup 0.12.0-dev1.1 [16]
 - Fix type mismatches in deser structs 0.12.0-dev2 [17]
 - Add -E flag for file paths 0.12.0-dev2.1 [18]
 - Add summary record variant 0.12.0-dev3 [19]
 - Separate empty files from skipped 0.12.0-dev3.1 [20]
 - cli flag cleanup 0.12.0-dev3.2 [21]
 - Fix all deserialization errors 0.12.0 [22]
 - Replace serde_json::Value with Untyped and typed structs 0.13.0 [23]
 - Label and indent -v,--valid like the others 0.13.1 [26]
 - Add error test data and improve error output 0.14.0 [24],[25]

Keep [a] here for on going testing:
 - Test 3 leading number signs and one # embedded [a]

[a]: chores-01.md#test-3-leading-number-signs-and-one--embedded

[1]: chores-01.md#have-claude-code-design-a-claude-code-session-viewer
[2]: chores-01.md#define-serde-structs-for-jsonl-deserialization-010
[3]: chores-01.md#add-new-record-type-variants-020
[4]: chores-01.md#refactor-common-metadata-fields-into-a-shared-sessionmetadata-struct
[5]: chores-01.md#support-text-blocks-in-user-content-arrays-030
[6]: chores-01.md#make-systemrecord-subtype-specific-fields-optional-040
[7]: chores-01.md#add-clap-cli-050
[8]: chores-01.md#add-all-optional-fields-seen-test-060
[9]: chores-01.md#compact-single-line-output-with-grouped-errors-070
[10]: chores-01.md#add-unknown-fields-from-vc-x1-sessions-080
[11]: chores-01.md#add-cli-flags-list-errors-recursive-glob-090
[12]: chores-01.md#add-support-for-agent-session-files
[13]: chores-01.md#add-agent-metajson-support-0100
[14]: chores-01.md#add-agentid-to-record-structs-for-agent-jsonl-0110
[15]: chores-01.md#add-new-optional-fields-0120-dev1
[16]: chores-01.md#sniff-test-and-exit-code-cleanup-0120-dev11
[17]: chores-01.md#fix-type-mismatches-in-deser-structs-0120-dev2
[18]: chores-01.md#add--e-flag-for-error-file-paths-0120-dev21
[19]: chores-01.md#add-summary-record-variant-0120-dev3
[20]: chores-01.md#separate-empty-files-from-skipped-0120-dev31
[21]: chores-01.md#cli-flag-cleanup-0120-dev32
[22]: chores-01.md#fix-all-deserialization-errors-0120
[23]: chores-01.md#replace-serde_jsonvalue-with-typed-structs
[24]: chores-01.md#add-error-test-data-and-library-tests
[25]: chores-01.md#improve-error-output-format-columnization-full-paths
[26]: chores-01.md#label-and-indent--v--valid-like-the-others