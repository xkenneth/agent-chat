use std::path::Path;
use crate::error::Result;
use crate::storage::{config, identity, lockfile, paths};
use crate::ui;

pub fn acquire(root: &Path, glob: &str) -> Result<()> {
    let id = identity::resolve(root)?;
    let name = identity::require_name(&id)?;

    let config = config::read_config(&paths::config_path(root))?;
    let locks_dir = paths::locks_dir(root);

    lockfile::acquire(&locks_dir, glob, name, &id.session_id, config.lock_ttl_secs)?;
    println!("{}", ui::success_line("Locked:", glob));
    Ok(())
}

pub fn release(root: &Path, glob: &str) -> Result<()> {
    let id = identity::resolve(root)?;

    let locks_dir = paths::locks_dir(root);
    lockfile::release(&locks_dir, glob, &id.session_id)?;
    println!("{}", ui::success_line("Unlocked:", glob));
    Ok(())
}

pub fn list(root: &Path) -> Result<()> {
    let locks_dir = paths::locks_dir(root);
    let locks = lockfile::list_active(&locks_dir)?;

    if locks.is_empty() {
        println!("{}", ui::info_line("Locks:", "No active locks."));
        return Ok(());
    }

    println!("{}", ui::table_header("PATTERN", "OWNER", Some("TTL")));
    for lock in &locks {
        let remaining = (lock.acquired_at + lock.ttl_secs).saturating_sub(
            std::time::SystemTime::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        println!("{:<30} {:<15} {}s", lock.glob, lock.owner, remaining);
    }
    Ok(())
}
