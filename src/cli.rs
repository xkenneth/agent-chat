use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "agent-chat", about = "File-based inter-agent communication")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create .agent-chat/ directory and install Claude Code hooks
    Init,

    /// Assign session identity (reads stdin JSON from hook)
    Register,

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
}
