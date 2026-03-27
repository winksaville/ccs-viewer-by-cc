# Chores-02

Session display and viewer UI. Builds on the validation/summary
foundation from [chores-01](chores-01.md) to actually render
session content.

See [Chores format](README.md#chores-format)

## Session display roadmap

The current tool validates and summarizes JSONL files but doesn't
show the conversation. The goal is to evolve from CLI transcript
to TUI to full GUI, each phase building on the data flow from the
previous one. The long-term vision is a tool that tells the
coherent "story" of code development by linking conversations to
the commits they produced.

### What a session contains

A session is a conversation tree:

- `user` / `assistant` exchanges linked by `parentUuid`
- Assistant responses contain `thinking`, `text`, and `tool_use` blocks
- `progress` records interleave (tool execution, agent spawns,
  commands, searches)
- `system` records mark API errors, retries, context compaction
- `file-history-snapshot` records track file state at each point
- Agents create sub-conversations (linked by `agentId`)

A typical session (461 records) breaks down as: 133 user +
182 assistant + 95 progress + 36 file-history-snapshot + 8 system +
misc. The "story" is buried in the interleaving.

### Phase 1: CLI transcript mode

Stay in the current binary. Add a `--show` / `--transcript` flag
that renders a readable conversation to stdout.

- Color-coded roles (user=green, assistant=blue, system=yellow)
- Show text content inline
- Summarize tool_use as one-liners (`[tool: Read src/main.rs]`)
- Collapse or hide thinking blocks by default (`--thinking` to expand)
- Skip progress/file-history-snapshot by default (`--all` to include)
- Pipe through a pager automatically when stdout is a tty

This forces us to figure out the right data flow (filtering,
ordering, collapsing) before investing in UI framework choices.
Zero new deps beyond ANSI codes.

### Phase 2: TUI with ratatui

ratatui is the clear choice in Rust -- actively maintained, rich
widget set, used by major tools (gitui, bottom, etc.).

Layout concept:

```
+-- Sessions --------+-- Conversation ----------------------------+
| session-31ba...    | [USER] reqaquaint                          |
| session-997a...    |                                            |
| session-092d...    | [ASSISTANT] Let me look at the project     |
|                    |   [tool: Read Cargo.toml]                  |
|                    |   [tool: Glob **/*.rs]                     |
|                    |                                            |
|                    | [USER] [tool_result: 14 lines]             |
|                    |                                            |
|                    | [ASSISTANT] This is a Rust project...      |
+--------------------+--------------------------------------------+
| Agents             | Metadata / Details                         |
|  Explore (3)       | model: claude-opus-4-6                     |
|  Plan (1)          | tokens: 1234 in / 567 out                  |
|                    | timestamp: 2026-03-21T21:19:13Z            |
+--------------------+--------------------------------------------+
```

- Left: session/file browser + agent tree
- Main: scrollable conversation with expand/collapse per block
- Bottom: metadata panel for selected record
- Keybinds: j/k scroll, Enter expand, tab switch panes, / search

### Phase 3: the "story" view

Cross-reference sessions with git history via ochid trailers:

- Show which conversation produced which commits
- Timeline view: code changes interleaved with the conversations
  that drove them
- Diff view: see what the assistant actually changed alongside
  the discussion

### GUI framework options (for later)

| Framework   | Style                       | Pros                                            | Cons                        |
|-------------|-----------------------------|-------------------------------------------------|-----------------------------|
| **ratatui** | Terminal                    | Zero runtime deps, fast, works over SSH         | No images, limited layout   |
| **egui**    | Immediate-mode GUI          | Very easy to prototype, cross-platform          | Not native-looking          |
| **iced**    | Elm-style GUI               | Native feel, good for complex layouts           | Steeper learning curve      |
| **tauri**   | Web frontend + Rust backend | Full HTML/CSS, richest rendering                | Heavyweight, needs JS       |

Recommendation: Phase 1 first (immediately useful, small delta),
then Phase 2 with ratatui (TUI fits session-browsing well, keeps
the tool lightweight). GUI frameworks for Phase 3+ if needed.
