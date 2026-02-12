use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use globset::{Glob, GlobMatcher};
use serde::{Deserialize, Serialize};

use crate::error::{AgentChatError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct LockEntry {
    pub glob: String,
    pub owner: String,
    pub session_id: String,
    pub acquired_at: u64, // unix epoch seconds
    pub ttl_secs: u64,
}

impl LockEntry {
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.acquired_at + self.ttl_secs
    }
}

/// Hash a glob pattern to create a stable filename.
fn hash_glob(glob: &str) -> String {
    let mut hasher = DefaultHasher::new();
    glob.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn lock_path(locks_dir: &Path, glob: &str) -> PathBuf {
    locks_dir.join(format!("{}.lock", hash_glob(glob)))
}

/// Acquire a lock on a glob pattern.
pub fn acquire(
    locks_dir: &Path,
    glob: &str,
    owner: &str,
    session_id: &str,
    ttl_secs: u64,
) -> Result<()> {
    // Clean expired locks first
    cleanup_expired(locks_dir)?;

    // Check for existing lock
    let path = lock_path(locks_dir, glob);
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        if let Ok(existing) = serde_json::from_str::<LockEntry>(&content) {
            if !existing.is_expired() {
                if existing.session_id == session_id {
                    // Re-acquiring own lock is OK, refresh it
                } else {
                    return Err(AgentChatError::LockConflict {
                        glob: glob.to_string(),
                        owner: existing.owner.clone(),
                    });
                }
            }
        }
    }

    let entry = LockEntry {
        glob: glob.to_string(),
        owner: owner.to_string(),
        session_id: session_id.to_string(),
        acquired_at: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        ttl_secs,
    };

    let content = serde_json::to_string_pretty(&entry)?;
    let tmp = locks_dir.join(format!(".tmp.{}", hash_glob(glob)));
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &path)?;
    Ok(())
}

/// Release a lock on a glob pattern. Only the owner session can release.
pub fn release(locks_dir: &Path, glob: &str, session_id: &str) -> Result<()> {
    let path = lock_path(locks_dir, glob);
    if !path.exists() {
        return Err(AgentChatError::LockNotFound(glob.to_string()));
    }

    let content = fs::read_to_string(&path)?;
    let entry: LockEntry = serde_json::from_str(&content)?;

    if entry.session_id != session_id && !entry.is_expired() {
        return Err(AgentChatError::LockConflict {
            glob: glob.to_string(),
            owner: entry.owner,
        });
    }

    // Ignore ENOENT race
    let _ = fs::remove_file(&path);
    Ok(())
}

/// List all active (non-expired) locks.
pub fn list_active(locks_dir: &Path) -> Result<Vec<LockEntry>> {
    let mut locks = Vec::new();
    if !locks_dir.exists() {
        return Ok(locks);
    }

    for entry in fs::read_dir(locks_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".lock") || name.starts_with(".tmp.") {
            continue;
        }
        match fs::read_to_string(entry.path()) {
            Ok(content) => {
                if let Ok(lock) = serde_json::from_str::<LockEntry>(&content) {
                    if !lock.is_expired() {
                        locks.push(lock);
                    } else {
                        // Clean up expired
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
            Err(_) => continue, // skip corrupt files
        }
    }
    Ok(locks)
}

/// Check if a file path matches any active lock NOT owned by the given session.
/// Returns the matching lock entry if found.
pub fn check_file(locks_dir: &Path, file_path: &str, session_id: &str) -> Result<Option<LockEntry>> {
    let locks = list_active(locks_dir)?;
    for lock in locks {
        if lock.session_id == session_id {
            continue; // own lock
        }
        if let Ok(glob) = Glob::new(&lock.glob) {
            let matcher: GlobMatcher = glob.compile_matcher();
            if matcher.is_match(file_path) {
                return Ok(Some(lock));
            }
        }
    }
    Ok(None)
}

/// Clean up expired lock files.
fn cleanup_expired(locks_dir: &Path) -> Result<()> {
    if !locks_dir.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(locks_dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.ends_with(".lock") || name.starts_with(".tmp.") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if let Ok(lock) = serde_json::from_str::<LockEntry>(&content) {
                if lock.is_expired() {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn acquire_and_list() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();
        let locks = list_active(tmp.path()).unwrap();
        assert_eq!(locks.len(), 1);
        assert_eq!(locks[0].glob, "src/*.rs");
        assert_eq!(locks[0].owner, "swift-fox");
    }

    #[test]
    fn acquire_conflict() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();
        let result = acquire(tmp.path(), "src/*.rs", "bold-hawk", "sess2", 300);
        assert!(result.is_err());
    }

    #[test]
    fn acquire_same_session_ok() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();
    }

    #[test]
    fn different_patterns_ok() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();
        acquire(tmp.path(), "tests/*.rs", "bold-hawk", "sess2", 300).unwrap();
        let locks = list_active(tmp.path()).unwrap();
        assert_eq!(locks.len(), 2);
    }

    #[test]
    fn release_lock() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();
        release(tmp.path(), "src/*.rs", "sess1").unwrap();
        let locks = list_active(tmp.path()).unwrap();
        assert_eq!(locks.len(), 0);
    }

    #[test]
    fn check_file_match() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 300).unwrap();

        // Different session should see the lock
        let result = check_file(tmp.path(), "src/main.rs", "sess2").unwrap();
        assert!(result.is_some());

        // Same session should not
        let result = check_file(tmp.path(), "src/main.rs", "sess1").unwrap();
        assert!(result.is_none());

        // Non-matching path
        let result = check_file(tmp.path(), "tests/foo.rs", "sess2").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn glob_matching_recursive() {
        let tmp = TempDir::new().unwrap();
        acquire(tmp.path(), "src/**/*.rs", "swift-fox", "sess1", 300).unwrap();
        let result = check_file(tmp.path(), "src/commands/init.rs", "sess2").unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn expired_lock_cleaned_up() {
        let tmp = TempDir::new().unwrap();
        // Create a lock with 0 TTL (immediately expired)
        acquire(tmp.path(), "src/*.rs", "swift-fox", "sess1", 0).unwrap();

        // Should be cleaned up on next list
        std::thread::sleep(std::time::Duration::from_millis(1100));
        let locks = list_active(tmp.path()).unwrap();
        assert_eq!(locks.len(), 0);
    }
}
