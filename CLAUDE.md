# CLAUDE.md - Bot Instructions

## Memory vs CLAUDE.md

Prefer CLAUDE.md for project instructions and conventions — it is
checked into the repo and portable. Use memory/ only for user-specific
context that does not belong in the codebase.

## Project Structure

This project uses **two separate jj-git repos**:

1. **App repo** (`/` — project root): Contains the application source code.
2. **Bot session repo** (`/.claude/`): Contains Claude Code session data.

Both repos are managed with `jj` (Jujutsu), which coexists with git.

## Repo Paths (relative from project root)

- App repo: `.` (project root)
- Bot session repo: `.claude`
  (symlink from `~/.claude/projects/<path-to-project-root>/.claude`)

## Working Directory

Prefer staying in the project root. Use `-R` flags or absolute paths
to target other directories rather than `cd`. If `cd` seems necessary,
discuss with the user first — losing track of cwd causes subtle
command failures downstream.

## Committing

Use `-R` (`--repository`) at the end to target the correct repo. Use
relative paths to reduce noise. Putting `-R` last keeps the verb/action
visible at the start of the command.

### App repo
```
jj commit -m \
"title" \
-m "body

ochid: /.claude/<changeID>" \
-R .
```

### Bot session repo
```
jj commit -m \
"title" \
-m "body

ochid: /<changeID>" \
-R .claude
```

## jj Basics

- `jj st -R .` / `jj st -R .claude` — show working copy status
- `jj log -R .` / `jj log -R .claude` — show commit log
- `jj commit -m "title" -m "body" -R <repo>` — finalize working copy into a commit
- `jj describe -m "title" -m "body" -R <repo>` — set description without committing
- `jj git push --bookmark <name> -R <repo>` — push a bookmark (no
  `--allow-new` flag; jj pushes new bookmarks without special flags)
- In jj, the working copy (@) is always a mutable commit being edited.
  `jj commit` finalizes it and creates a new empty working copy on top.
- The `.claude` repo always has uncommitted changes during an active
  session because session data updates continuously.

## Commit Message Style

Use [Conventional Commits](https://www.conventionalcommits.org/) with
a version suffix:

```
<type>: <short description> (<version>)
```

- **Title**: max 52 chars, short summary of *what* changed.
  Include the version. Common types: `feat`, `fix`, `refactor`,
  `test`, `docs`, `chore`.
- **Body**: max 72 chars per line. Start with a short explanatory
  sentence or paragraph, then a blank line, then a bulleted list
  of changes. Keep items as readable sentences.
- Examples:
  - `feat: add fix-ochid subcommand (0.22.0)`
  - `fix: fix-ochid prefix bug (0.22.1)`
  - `refactor: deduplicate common CLI flags (0.21.1)`

## Pre-commit Requirements

### User approval

Never execute commit, squash, push, or finalize commands without the
user's explicit approval. Present changes for review first; only run
them after the user confirms. This applies to late changes too —
pause for review before squashing into an existing commit.

### Notes references

Multiple references must be separated: `[2],[3]` not `[2,3]` or `[2][3]`.
See [Todo format](notes/README.md#todo-format) for details.

### Versioning

Every change must start with a version bump. See
[Versioning during development](notes/README.md#versioning-during-development)
for details. Get user approval on single-step vs multi-step before starting.

### Pre-commit checklist

Before proposing a commit, run all of the following and fix any issues:

1. `cargo fmt`
2. `cargo clippy`
3. `cargo test`
4. `cargo install --path .` (if applicable)
5. Retest after install:
   - `ccs-viewer data/ccs-viewer-tests.jsonl` (new test data)
   - `ccs-viewer data/*.jsonl` (regression check)
6. Update `notes/todo.md` — add to `## Done` if completing a task
7. Update `notes/chores-*.md` — add a subsection describing the change
8. Update `notes/README.md` — if functionality changed (new flags,
   new subcommands, changed behavior)

## ochid Trailers

Every commit body must include an `ochid:` trailer pointing to the
counterpart commit in the other repo. The value is a workspace-root-relative
path followed by the **12-character** changeID (the full short form from
`vc-x1 chid -L`, not the truncated prefix jj shows in logs):

- App repo commits point to `.claude`: `ochid: /.claude/<changeID>`
- Bot session commits point to app repo: `ochid: /<changeID>`

Use `vc-x1 chid -R .,.claude -L` to get both 12-character changeIDs
(first line is app repo, second is `.claude`).

## Commit and Push Workflow

When the user asks to commit, follow this sequence:

1. **Get changeIDs first** — run `vc-x1 chid -R .,.claude -L` before
   presenting commit commands so ochid trailers are filled in (DRY).
2. **Present both commits** for user review with ochids already
   populated. Use the **same title** for both commits. The body can
   differ: app repo summarizes code changes; bot session notes what
   was done.
3. **Wait for user approval** before executing anything.
4. **Commit both repos** after approval.
5. **Advance bookmarks** on both repos (`jj bookmark set <bookmark>
   -r @- -R .` / `-R .claude`).
6. **Only push if the user asked to push.** Do **not** push `.claude`
   — `finalize` handles that.

# Step 1: get changeIDs
```
vc-x1 chid -R .,.claude -L
```

# Step 2: prepare commit and present to user
```
jj commit -m "shared title" -m "app body\n\nochid: /.claude/<claude-chid>" -R .
jj commit -m "shared title" -m "session body\n\nochid: /<app-chid>" -R .claude
```

# Step 3: Ask for approval to commit

# Step 4: Advance bookmarks
```
jj bookmark set <bookmark> -r @- -R .
jj bookmark set <bookmark> -r @- -R .claude
```

# Step 5: Ask for approval to push and finalize if requested see [Finalize the .claude repo](CLAUDE.md#Finalize-the--claude-repo)

Manually push the app repo, `-R .`, do **not** push `.claude` use
`finalize` instead, which handles squashing @ to @- and pushes .claude
```
jj git push --bookmark <bookmark> -R .
vc-x1 finalize --repo .claude --bookmark <bookmark> --delay 10 --detach --push
```

### Late changes after push

If changes are made to the app repo after it has been pushed (e.g.
updating CLAUDE.md or memory), the commit is now immutable. Use
`--ignore-immutable` to squash the changes in, then re-push:

```
jj squash --ignore-immutable -R .
jj bookmark set <bookmark> -r @- -R .
jj git push --bookmark <bookmark> -R .
```

### Finalize the .claude repo

The **very last action** in a session is to finalize the `.claude` repo.
This squashes the working copy into the session commit and pushes. The
delay gives a safety margin against any pending writes. Always use a
short relative path for `--repo`.

**Nothing should happen after finalize** — no memory writes, no tool
calls, no additional output. If any work is done after finalize, run
finalize again so the trailing writes are captured.

`--bookmark` is required — use the active bookmark for the session.

```
vc-x1 finalize --repo .claude --bookmark <bookmark> --delay 10 --detach --push
```

Do **not** echo or restate the finalize output — the Bash tool
already displays it. Any trailing text output creates writes that
miss the finalize squash window.
