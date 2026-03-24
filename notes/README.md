# Notes

This directory contains various notes and documentation related to the project.
Each file is organized by topic for easy reference.

## CLI Usage

```
ccs-viewer [OPTIONS] <PATTERNS>...
```

By default, only the summary line is printed. Use flags for more detail.

### Options

| Flag | Short | Description |
|------|-------|-------------|
| `--list` | `-l` | Show per-file summary lines |
| `--errors` | `-e` | Show grouped error details after summary |
| `--recursive` | `-r` | Treat positional args as directories, search recursively |
| `--glob <PAT>` | | File pattern for recursive mode (repeatable, default: `*.jsonl`) |
| `--version` | `-V` | Print version |
| `--help` | `-h` | Print help |

### Positional arguments

Without `-r`: file glob patterns expanded by the program.
With `-r`: directory paths (or directory globs) to search recursively.

### Examples

```
# Summary only (default)
ccs-viewer "data/*.jsonl"

# Per-file list + summary
ccs-viewer -l "data/*.jsonl"

# Summary + error details
ccs-viewer -e "data/*.jsonl"

# Recursive search in a directory
ccs-viewer -r .claude

# Recursive with custom file patterns
ccs-viewer -r --glob "*.jsonl" --glob "*.json" .claude

# Multiple directories
ccs-viewer -r .claude ../vc-x1/.claude

# Directory glob
ccs-viewer -r "../*/.claude"

# All flags
ccs-viewer -r -l -e .claude
```

### Exit code

Exits 0 on success, 1 if any deserialization errors occurred.

By default there are chores-*.md and todo.md. Chores are general notes
about tasks and todo.md contains short term tasks and their status.

In the future we I expect we may want to create a "notes"
database to better manage the information, TBD.

Examples chore file:
```
# Chores-01.md
 
General maintenance tasks and considerations for the project see other files for
more specific topics. A chore in a chores file provides quick information on the
how and why of a particular chore.

## Create a binary that lists jj info 

This binary should list the changeID, commitID, and description title
and using `jj-lib`
```

## jj tips

For users new to jj see [jj-tips.md](jj-tips.md).

```
## Chores format

Filename: "Chores-XX-.md"
example: chores-01.md

Format of section labels: "## <short description> (YYYYMMDD X.Y.Z)"
example: "## Topic format description (20260322 0.1.0)"

Example chore file:
```
# Chores-01.md
 
General maintenance tasks and considerations for the project see other files for
more specific topics. A chore in a chores file provides quick information on the
how and why of a particular chore.

## Do something (20260322 1.3.1)

Describe something
```

## Versioning during development

This is using jujustiu, jj + git and we'll see how it goes. Below is my
git workflow, jj will be different but we'll have to discover that as
we go.

Every plan must start with a version bump. Choose the approach based on scope:

- **Single-step** (recommended for mechanical/focused changes): bump directly to
  `X.Y.Z`, implement in one commit. Simpler history.
- **Multi-step** (for exploratory/large changes): bump to `X.Y.Z-devN`, implement
  across multiple commits, final commit removes `-devN`.

The plan should recommend one approach and get user approval before starting.

For multi-step:
1. Bump version to `X.Y.Z-devN` with a plan and commit as a chore marker
2. Implement in one or more `-devN` commits (bump N as needed)
3. Final commit removes `-devN`, updates todo/chores — this is the "done" marker

The final release commit (without `-devN`) signals completion rather than amending
prior commits. This keeps the git history readable and makes it easy to see which
commits were exploratory vs final.

## Todo format

Todo.md contains two main sections "Todo" and "Done" each item is a
short explanations of a tasks and links to more details using 1 or more
references.

Multiple references must be separated: `[2],[3]` not `[2,3]` or `[2][3]`.
In markdown, `[2,3]` is a single ref key (won't resolve) and `[2][3]`
is parsed as display text `2` with ref key `3` (so `[2]` won't resolve).

Examples:

# Todo
- Add new feature X [details](features.md#feature-x)
- Fix bug Y [1]

# Done
- Fixed issue Z [2],[3]

[1]: chores-01bugs.md#bug-y
