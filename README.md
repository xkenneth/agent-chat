# agent-chat

File-based chatroom for Claude Code and Codex agents. No server, no database, no MCP protocol — just flat files and a single static binary.

Multiple agent sessions working on the same project can coordinate through a shared message log with advisory file locking. `agent-chat init` can install Claude integration, Codex integration, or both.

## Quick start

```bash
cargo install --path .

# In your project directory:
agent-chat init
```

That's it. `init` creates the `.agent-chat/` directory and can install Claude integration, Codex integration, or both. Without flags it prompts interactively:

```
Agent Chat setup
Choose integration(s):
  [1] Claude (hooks + CLAUDE.md)
  [2] Codex  (AGENTS.md)
  [3] Both (default)
Select 1/2/3 (Enter = default) >
```

Second prompt default is `User` install target when you press Enter.

Or use flags, e.g. `agent-chat init --project --claude`, `agent-chat init --project --codex`, or `agent-chat init --project --both-tools`.

For Codex guidance (AGENTS.md instead of Claude hooks):

```bash
agent-chat init-codex --project
agent-chat register --session-id "<your-session-id>"
```

For mixed projects (Claude + Codex in the same repo), use:

```bash
agent-chat init --project --both-tools
```

The next time Claude Code starts a session in this project, it will auto-register with a friendly name like `swift-fox` and begin checking for messages.

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

### Core

| Command | Purpose | Stdout |
|---------|---------|--------|
| `init [--project\|--user\|--both] [--claude\|--codex\|--both-tools]` | Create `.agent-chat/`, install selected integration(s) | Setup confirmation |
| `register [--session-id <id>]` | Assign session identity (stdin JSON for hooks, or explicit id) | `You are swift-fox...` |
| `say <msg>` | Post to shared log | Nothing |
| `read [--all]` | Show unread (or all) messages, advance cursor | Messages only |
| `status` | Unread check for Stop hook | `[agent-chat: N unread]` or nothing |
| `lock <glob>` | Advisory file lock with TTL | Confirmation |
| `unlock <glob>` | Release lock | Confirmation |
| `locks` | List active locks | Table |
| `check-lock` | PreToolUse hook (Edit/Write), reads stdin JSON | Warning JSON or nothing |
| `check-messages` | PreToolUse hook (Bash), injects unread messages | `additionalContext` JSON or nothing |

### Beads (br) integration

| Command | Purpose | Stdout |
|---------|---------|--------|
| `init-br [--project\|--user]` | Install br guidance into `CLAUDE.md` | Setup confirmation |
| `br-claim <id>` | Set issue to `in_progress`, assign self, announce | Nothing |
| `br-complete <id> [--reason R]` | Close issue, announce completion | Nothing |
| `init-codex [--project\|--user\|--both]` | Install Codex guidance into `AGENTS.md` | Setup confirmation |

## Claude + Codex compatibility

- Both tools share the same `.agent-chat/` state (messages, sessions, cursors, locks, focuses).
- Claude sessions auto-register via hooks and get env wiring from `CLAUDE_ENV_FILE`.
- Codex sessions register with `agent-chat register --session-id <id>`.
- After registration, commands resolve identity with env-first semantics:
  - If env is present, use it.
  - If env is missing, use `.agent-chat/sessions/<session_id>`.
  - If `session_id` is missing and exactly one session exists, infer it automatically.

## Hooks

Installed automatically by `init` into `.claude/settings.local.json` (project) or `~/.claude/settings.json` (user):

- **SessionStart** — `agent-chat register` reads the session JSON from stdin, generates a friendly name (e.g. `swift-fox`), writes `AGENT_CHAT_NAME` and `AGENT_CHAT_SESSION_ID` to `$CLAUDE_ENV_FILE` so identity survives context compaction, and injects any unread messages.
- **Stop** — `agent-chat status` returns `{"decision": "block", "reason": "..."}` if there are unread messages, preventing the agent from stopping until it reads them. Returns nothing (zero tokens) when all caught up.
- **PreToolUse** (Edit|Write) — `agent-chat check-lock` checks if the target file matches another agent's lock and returns a `hookSpecificOutput` warning if so.
- **PreToolUse** (Bash) — `agent-chat check-messages` injects unread messages as `additionalContext` before bash commands, so agents stay aware of other agents' activity without explicit `read` calls.

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
