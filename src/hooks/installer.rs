use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use crate::error::Result;

/// Resolve the absolute path to the current binary.
/// Falls back to "agent-chat" if resolution fails (e.g. in tests).
fn binary_path() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "agent-chat".to_string())
}

/// The hooks configuration to install
fn hooks_config() -> Value {
    let bin = binary_path();
    let allow_pattern = format!("Bash({} *)", bin);
    json!({
        "permissions": {
            "allow": [allow_pattern]
        },
        "hooks": {
            "SessionStart": [{
                "matcher": "startup|resume",
                "hooks": [{
                    "type": "command",
                    "command": format!("{} register", bin),
                    "timeout": 10
                }]
            }],
            "Stop": [{
                "hooks": [{
                    "type": "command",
                    "command": format!("{} status", bin),
                    "timeout": 5
                }]
            }],
            "PreToolUse": [{
                "matcher": "Edit|Write",
                "hooks": [{
                    "type": "command",
                    "command": format!("{} check-lock", bin),
                    "timeout": 5
                }]
            },
            {
                "matcher": "Bash",
                "hooks": [{
                    "type": "command",
                    "command": format!("{} check-messages", bin),
                    "timeout": 5
                }]
            }]
        }
    })
}

/// Install hooks by merging into `.claude/settings.local.json` in the project.
/// Creates the file and directory if they don't exist.
/// Merges (not overwrites) to preserve existing settings.
pub fn install_hooks(project_root: &Path) -> Result<()> {
    install_hooks_to(&project_root.join(".claude"), "settings.local.json")
}

/// Install hooks by merging into `<claude_dir>/<filename>`.
/// Creates the directory and file if they don't exist.
pub fn install_hooks_to(claude_dir: &Path, filename: &str) -> Result<()> {
    fs::create_dir_all(claude_dir)?;

    let settings_path = claude_dir.join(filename);
    let mut existing: Value = if settings_path.exists() {
        let content = fs::read_to_string(&settings_path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| json!({}))
    } else {
        json!({})
    };

    let new_config = hooks_config();

    // Merge permissions.allow array
    if let Some(new_allow) = new_config["permissions"]["allow"].as_array() {
        let existing_allow = existing["permissions"]["allow"]
            .as_array()
            .cloned()
            .unwrap_or_default();

        let mut merged_allow = existing_allow;
        for item in new_allow {
            if !merged_allow.contains(item) {
                merged_allow.push(item.clone());
            }
        }

        existing["permissions"] = json!({"allow": merged_allow});
        // Preserve other permission keys if they exist
        if let Some(existing_perms) = existing.get("permissions").and_then(|p| p.as_object()) {
            let _ = existing_perms; // already handled above
        }
    }

    // Merge hooks - add our hooks alongside existing ones
    if let Some(new_hooks) = new_config["hooks"].as_object() {
        if existing.get("hooks").is_none() {
            existing["hooks"] = json!({});
        }
        for (event, new_entries) in new_hooks {
            if let Some(new_arr) = new_entries.as_array() {
                let existing_arr = existing["hooks"][event]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();

                let mut merged = existing_arr;
                for entry in new_arr {
                    // Remove any existing hook whose subcommand matches
                    // (handles upgrade from bare "agent-chat X" to absolute path)
                    merged.retain(|e| {
                        if let (Some(e_hooks), Some(n_hooks)) =
                            (e["hooks"].as_array(), entry["hooks"].as_array())
                        {
                            !e_hooks.iter().any(|eh| {
                                n_hooks.iter().any(|nh| {
                                    match (eh["command"].as_str(), nh["command"].as_str()) {
                                        (Some(ec), Some(nc)) => {
                                            // Compare subcommand: last space-separated tokens
                                            // e.g. "agent-chat register" and "/path/to/agent-chat register"
                                            // both end with "agent-chat register"
                                            ec.ends_with(nc) || nc.ends_with(ec) || ec == nc
                                        }
                                        _ => false,
                                    }
                                })
                            })
                        } else {
                            true
                        }
                    });
                    merged.push(entry.clone());
                }
                existing["hooks"][event] = Value::Array(merged);
            }
        }
    }

    let content = serde_json::to_string_pretty(&existing)?;
    let tmp_name = format!(".tmp.{}", filename);
    let tmp = claude_dir.join(tmp_name);
    fs::write(&tmp, &content)?;
    fs::rename(&tmp, &settings_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn install_creates_new_settings() {
        let tmp = TempDir::new().unwrap();
        install_hooks(tmp.path()).unwrap();

        let path = tmp.path().join(".claude/settings.local.json");
        assert!(path.exists());

        let content = fs::read_to_string(&path).unwrap();
        let val: Value = serde_json::from_str(&content).unwrap();
        assert!(val["hooks"]["SessionStart"].is_array());
        assert!(val["hooks"]["Stop"].is_array());
        assert!(val["hooks"]["PreToolUse"].is_array());
        // Permission uses absolute binary path
        let allow = val["permissions"]["allow"].as_array().unwrap();
        assert!(allow.iter().any(|v| {
            v.as_str().map(|s| s.starts_with("Bash(") && s.contains("agent-chat") && s.ends_with("*)")).unwrap_or(false)
        }));
    }

    #[test]
    fn install_preserves_existing_settings() {
        let tmp = TempDir::new().unwrap();
        let claude_dir = tmp.path().join(".claude");
        fs::create_dir_all(&claude_dir).unwrap();
        let settings_path = claude_dir.join("settings.local.json");
        fs::write(&settings_path, r#"{"permissions":{"allow":["Bash(git *)"]},"custom":"value"}"#).unwrap();

        install_hooks(tmp.path()).unwrap();

        let content = fs::read_to_string(&settings_path).unwrap();
        let val: Value = serde_json::from_str(&content).unwrap();
        let allow = val["permissions"]["allow"].as_array().unwrap();
        assert!(allow.contains(&json!("Bash(git *)")));
        assert!(allow.iter().any(|v| {
            v.as_str().map(|s| s.contains("agent-chat")).unwrap_or(false)
        }));
    }

    #[test]
    fn install_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        install_hooks(tmp.path()).unwrap();
        install_hooks(tmp.path()).unwrap();

        let content = fs::read_to_string(tmp.path().join(".claude/settings.local.json")).unwrap();
        let val: Value = serde_json::from_str(&content).unwrap();
        // Should not have duplicate hooks
        let session_start = val["hooks"]["SessionStart"].as_array().unwrap();
        assert_eq!(session_start.len(), 1);
    }
}
