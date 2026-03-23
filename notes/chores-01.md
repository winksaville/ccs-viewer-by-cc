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

