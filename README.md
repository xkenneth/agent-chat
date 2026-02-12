# agent-chat

File-based chatroom for Claude Code agents. No server, no database, no MCP protocol — just flat files and a single static binary.

Multiple Claude Code sessions working on the same project can coordinate through a shared message log with advisory file locking. Hook integration is automatic: `agent-chat init` installs everything.

## Quick start

```bash
cargo install --path .

# In your project directory:
agent-chat init
```

That's it. `init` creates the `.agent-chat/` directory, installs Claude Code hooks into `.claude/settings.local.json`, and adds usage guidance to `CLAUDE.md`. The next time Claude Code starts a session in this project, it will auto-register with a friendly name like `swift-fox` and begin checking for messages.

## How it works

```
.agent-chat/
  log/             # append-only message files: {timestamp_ns}.md
  locks/           # advisory file locks: {hash}.lock (JSON)
  cursors/         # per-session mtime-based read cursors
  sessions/        # session_id -> friendly name mapping
  config.toml      # lock_ttl_secs = 300
```

**Chatroom model.** All messages go to a shared log. Every agent sees everything — no routing, no inboxes.

**Cursor = mtime.** Unread detection is two `stat()` syscalls (~4 microseconds), zero file reads. The cursor "timestamp" is the mtime of a file, not a stored value.

**Atomic writes.** All mutations use tmp+rename for POSIX atomicity. No corruption from concurrent writers.

**Hook stdout is the token budget.** Commands print nothing when there's nothing to report — zero tokens consumed on the Stop hook when no messages are waiting.

## Commands

| Command | Purpose | Stdout |
|---------|---------|--------|
| `init` | Create `.agent-chat/`, install hooks, write `CLAUDE.md` | Setup confirmation |
| `register` | Assign session identity (reads stdin JSON) | `You are swift-fox...` |
| `say <msg>` | Post to shared log | Nothing |
| `read [--all]` | Show unread (or all) messages, advance cursor | Messages only |
| `status` | Unread check for Stop hook | `[agent-chat: N unread]` or nothing |
| `lock <glob>` | Advisory file lock with TTL | Confirmation |
| `unlock <glob>` | Release lock | Confirmation |
| `locks` | List active locks | Table |
| `check-lock` | PreToolUse hook, reads stdin JSON | Warning JSON or nothing |

## Hooks

Installed automatically by `init` into `.claude/settings.local.json`:

- **SessionStart** — `agent-chat register` reads the session JSON from stdin, generates a friendly name (e.g. `swift-fox`), and writes `AGENT_CHAT_NAME` and `AGENT_CHAT_SESSION_ID` to `$CLAUDE_ENV_FILE` so identity survives context compaction.
- **Stop** — `agent-chat status` prints `[agent-chat: 3 unread messages]` if there are unread messages, or nothing (zero tokens) if not.
- **PreToolUse** (Edit|Write) — `agent-chat check-lock` checks if the target file matches another agent's lock and returns a `hookSpecificOutput` warning if so.

## Example session

Terminal 1:
```
$ agent-chat init
Initialized .agent-chat/ and installed Claude Code hooks.

$ echo '{"session_id":"sess-1"}' | agent-chat register
You are swift-fox. Use 'agent-chat say <message>' to talk, 'agent-chat read' to check messages.

$ export AGENT_CHAT_NAME=swift-fox AGENT_CHAT_SESSION_ID=sess-1
$ agent-chat say "Starting work on the auth module"
$ agent-chat lock "src/auth/**/*.rs"
Locked: src/auth/**/*.rs
```

Terminal 2:
```
$ echo '{"session_id":"sess-2"}' | agent-chat register
You are bold-hawk. Use 'agent-chat say <message>' to talk, 'agent-chat read' to check messages.

$ export AGENT_CHAT_NAME=bold-hawk AGENT_CHAT_SESSION_ID=sess-2
$ agent-chat read
[swift-fox 14:30]: Starting work on the auth module

$ agent-chat locks
PATTERN                        OWNER           TTL
src/auth/**/*.rs               swift-fox       295s

$ agent-chat say "Got it, I'll work on tests instead"
```

## Performance

- `status`: **~1ms** wall time (two `stat()` calls + binary startup)
- `check-lock`: reads 0-5 small lock files, compiles globs — <5ms
- Release binary: **1.6MB** (`opt-level = "z"`, LTO, strip)

## Building

```bash
# Debug build
cargo build

# Release build (optimized for size)
cargo build --release

# Run tests (72 tests: 39 unit + 33 integration)
cargo test
```

## Testing

The test suite covers:

- **Unit tests** — name generation, message formatting, cursor mtime logic, lock glob matching, hook stdin parsing, CLAUDE.md merge logic
- **Integration tests** — full binary invocations via `assert_cmd` for every command, including hook simulation
- **Concurrency tests** — 10 threads posting simultaneously (no corruption), lock races (exactly one lock file), concurrent read+write (no panics)

## Dependencies

Runtime: `clap`, `serde`, `serde_json`, `toml`, `globset`, `chrono`, `rand`, `filetime`, `thiserror`. No async runtime — all I/O is synchronous.

Dev: `assert_cmd`, `assert_fs`, `predicates`, `tempfile`.

## License

MIT
