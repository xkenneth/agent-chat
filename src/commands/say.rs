use std::path::Path;
use crate::error::Result;
use crate::storage::{identity, log, paths};

pub fn run(root: &Path, message: &str) -> Result<()> {
    let id = identity::resolve(root)?;
    let name = identity::require_name(&id)?;

    let log_dir = paths::log_dir(root);
    log::write_message(&log_dir, name, message)?;
    Ok(())
}
