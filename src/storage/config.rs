use serde::{Deserialize, Serialize};
use std::path::Path;
use crate::error::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_lock_ttl")]
    pub lock_ttl_secs: u64,
}

fn default_lock_ttl() -> u64 {
    300
}

impl Default for Config {
    fn default() -> Self {
        Config {
            lock_ttl_secs: default_lock_ttl(),
        }
    }
}

pub fn write_default_config(path: &Path) -> Result<()> {
    let config = Config::default();
    let content = toml::to_string_pretty(&config)?;
    std::fs::write(path, content)?;
    Ok(())
}

pub fn read_config(path: &Path) -> Result<Config> {
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
