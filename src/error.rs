use std::io;

#[derive(Debug, thiserror::Error)]
pub enum AgentChatError {
    #[error("Not initialized. Run 'agent-chat init'.")]
    NotInitialized,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Lock conflict: {glob} is locked by {owner}")]
    LockConflict { glob: String, owner: String },

    #[error("Lock not found: {0}")]
    LockNotFound(String),

    #[error("Missing environment variable: {0}")]
    MissingEnv(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, AgentChatError>;
