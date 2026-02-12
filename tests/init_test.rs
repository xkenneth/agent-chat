use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

#[test]
fn init_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();

    cmd()
        .arg("init")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialized"));

    assert!(tmp.path().join(".agent-chat/log").is_dir());
    assert!(tmp.path().join(".agent-chat/locks").is_dir());
    assert!(tmp.path().join(".agent-chat/cursors").is_dir());
    assert!(tmp.path().join(".agent-chat/sessions").is_dir());
    assert!(tmp.path().join(".agent-chat/config.toml").exists());
    assert!(tmp.path().join("CLAUDE.md").exists());
}

#[test]
fn init_writes_default_config() {
    let tmp = TempDir::new().unwrap();

    cmd().arg("init").current_dir(tmp.path()).assert().success();

    let config = std::fs::read_to_string(tmp.path().join(".agent-chat/config.toml")).unwrap();
    assert!(config.contains("lock_ttl_secs"));
}

#[test]
fn init_installs_hooks() {
    let tmp = TempDir::new().unwrap();

    cmd().arg("init").current_dir(tmp.path()).assert().success();

    let settings = std::fs::read_to_string(tmp.path().join(".claude/settings.local.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&settings).unwrap();
    assert!(val["hooks"]["SessionStart"].is_array());
    assert!(val["hooks"]["Stop"].is_array());
    assert!(val["hooks"]["PreToolUse"].is_array());
}

#[test]
fn init_preserves_existing_settings() {
    let tmp = TempDir::new().unwrap();
    let claude_dir = tmp.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join("settings.local.json"),
        r#"{"permissions":{"allow":["Bash(git *)"]}}"#,
    )
    .unwrap();

    cmd().arg("init").current_dir(tmp.path()).assert().success();

    let settings = std::fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&settings).unwrap();
    let allow = val["permissions"]["allow"].as_array().unwrap();
    assert!(allow.contains(&serde_json::json!("Bash(git *)")));
    assert!(allow.contains(&serde_json::json!("Bash(agent-chat *)")));
}

#[test]
fn init_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    cmd().arg("init").current_dir(tmp.path()).assert().success();
    cmd().arg("init").current_dir(tmp.path()).assert().success();

    let settings = std::fs::read_to_string(tmp.path().join(".claude/settings.local.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&settings).unwrap();
    let session_start = val["hooks"]["SessionStart"].as_array().unwrap();
    assert_eq!(session_start.len(), 1, "Should not duplicate hooks");

    let claude_md = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert_eq!(
        claude_md.matches("<!-- agent-chat:start -->").count(),
        1,
        "Should not duplicate CLAUDE.md section"
    );
}

#[test]
fn init_creates_claude_md() {
    let tmp = TempDir::new().unwrap();

    cmd().arg("init").current_dir(tmp.path()).assert().success();

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("# Agent Chat"));
    assert!(content.contains("agent-chat read"));
    assert!(content.contains("agent-chat say"));
    assert!(content.contains("agent-chat lock"));
}

#[test]
fn init_preserves_existing_claude_md() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("CLAUDE.md"), "# My Project\n\nDo not delete this.\n").unwrap();

    cmd().arg("init").current_dir(tmp.path()).assert().success();

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("# My Project"));
    assert!(content.contains("Do not delete this."));
    assert!(content.contains("# Agent Chat"));
}
