use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-chat", about = "File-based inter-agent communication")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create .agent-chat/ directory and install selected integrations
    Init {
        /// Install to project files (e.g. .claude/settings.local.json, CLAUDE.md, AGENTS.md)
        #[arg(long)]
        project: bool,
        /// Install to user files (e.g. ~/.claude/settings.json, ~/.claude/CLAUDE.md, ~/.codex/AGENTS.md)
        #[arg(long)]
        user: bool,
        /// Install to both project and user
        #[arg(long)]
        both: bool,
        /// Install Claude integration
        #[arg(long)]
        claude: bool,
        /// Install Codex integration
        #[arg(long)]
        codex: bool,
        /// Install both Claude and Codex integrations
        #[arg(long = "both-tools")]
        both_tools: bool,
    },

    /// Assign session identity (reads stdin JSON from hook, or use --session-id)
    Register {
        /// Explicit session ID (for environments without SessionStart hooks)
        #[arg(long)]
        session_id: Option<String>,
    },

    /// Install AGENTS.md guidance for Codex workflows
    InitCodex {
        /// Install to project (./AGENTS.md)
        #[arg(long)]
        project: bool,
        /// Install to user (~/.codex/AGENTS.md)
        #[arg(long)]
        user: bool,
        /// Install to both project and user
        #[arg(long)]
        both: bool,
    },

    /// Post a message to the shared log
    Say {
        /// Message text
        message: Vec<String>,
    },

    /// Show unread messages (or all with --all)
    Read {
        /// Show all messages instead of just unread
        #[arg(long)]
        all: bool,
    },

    /// Check for unread messages (for Stop hook)
    Status,

    /// Acquire an advisory file lock
    Lock {
        /// Glob pattern to lock
        glob: String,
    },

    /// Release an advisory file lock
    Unlock {
        /// Glob pattern to unlock
        glob: String,
    },

    /// List active locks
    Locks,

    /// Check if a file is locked (PreToolUse hook, reads stdin JSON)
    CheckLock,

    /// Nudge agent about unread messages (PreToolUse hook for Bash)
    CheckMessages,

    /// Install br (beads_rust) guidance into CLAUDE.md
    InitBr {
        /// Install to project (./CLAUDE.md)
        #[arg(long)]
        project: bool,
        /// Install to user (~/.claude/CLAUDE.md)
        #[arg(long)]
        user: bool,
    },

    /// Declare your focus area (or clear with --clear)
    Focus {
        /// Focus area text (e.g. "CI pipeline")
        text: Option<String>,
        /// Clear your current focus
        #[arg(long)]
        clear: bool,
    },

    /// List active agent focuses
    Focuses,

    /// Claim a br issue (sets in_progress + announces)
    BrClaim {
        /// Issue ID
        id: String,
    },

    /// Complete a br issue (closes + announces)
    BrComplete {
        /// Issue ID
        id: String,
        /// Optional reason for closing
        #[arg(long)]
        reason: Option<String>,
    },
}
