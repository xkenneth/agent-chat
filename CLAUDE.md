<!-- agent-chat:start -->
# Agent Chat

You are collaborating with other agents on this project. You were auto-registered
at session start — your name is in `$AGENT_CHAT_NAME`. Use it when referring to yourself.

## Commands

- `agent-chat say <msg>` — post to the shared chatroom
- `agent-chat read` — check for messages from other agents
- `agent-chat lock <glob>` — claim advisory file lock before editing
- `agent-chat unlock <glob>` — release when done
- `agent-chat locks` — see who's locked what

## Workflow

**Starting a task:**
1. Run `agent-chat read` to catch up on any messages
2. Say what you're about to work on: `agent-chat say "starting on auth middleware"`
3. Lock files you'll edit: `agent-chat lock "src/auth/**/*.rs"`

**While working:**
- Run `agent-chat read` every few tool calls — don't go more than 3-4 turns
  without checking. Other agents may be waiting on you or sharing info you need.
- Don't stop to wait for replies. If you've asked a question or are waiting on
  another agent, move to your next task.
- If the Stop hook shows unread messages, run `agent-chat read` immediately —
  do NOT stop without reading them first. Another agent may be blocked on you.

**Finishing a task:**
1. Unlock your files: `agent-chat unlock "src/auth/**/*.rs"`
2. Announce completion: `agent-chat say "auth middleware done, tests passing"`
3. Run `agent-chat read` to check if anything came in while you were working

**When blocked:**
- Say so: `agent-chat say "blocked on DB schema — need table layout from bold-hawk"`
- Move to a different task instead of waiting
- Run `agent-chat read` before starting the next task

## Message style

Keep messages short and actionable. Other agents pay tokens to read them.

- Good: `agent-chat say "lock conflict on src/api.rs — I'll take src/models.rs instead"`
- Bad: `agent-chat say "I noticed that the file src/api.rs appears to be locked by another agent, so I have decided to work on a different file instead, specifically src/models.rs"`

## File locking

Locks are advisory and expire after 5 minutes. Lock before multi-file edits,
unlock immediately when done. If `check-lock` warns you about a locked file,
coordinate with the lock owner before editing — don't just ignore the warning.
<!-- agent-chat:end -->