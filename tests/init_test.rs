use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

// ── --project (old default behavior) ────────────────────────────────

#[test]
fn init_project_creates_directory_structure() {
    let tmp = TempDir::new().unwrap();

    cmd()
        .args(["init", "--project"])
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
fn init_project_writes_default_config() {
    let tmp = TempDir::new().unwrap();

    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();

    let config = std::fs::read_to_string(tmp.path().join(".agent-chat/config.toml")).unwrap();
    assert!(config.contains("lock_ttl_secs"));
}

#[test]
fn init_project_installs_hooks() {
    let tmp = TempDir::new().unwrap();

    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();

    let settings = std::fs::read_to_string(tmp.path().join(".claude/settings.local.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&settings).unwrap();
    assert!(val["hooks"]["SessionStart"].is_array());
    assert!(val["hooks"]["Stop"].is_array());
    assert!(val["hooks"]["PreToolUse"].is_array());
}

#[test]
fn init_project_preserves_existing_settings() {
    let tmp = TempDir::new().unwrap();
    let claude_dir = tmp.path().join(".claude");
    std::fs::create_dir_all(&claude_dir).unwrap();
    std::fs::write(
        claude_dir.join("settings.local.json"),
        r#"{"permissions":{"allow":["Bash(git *)"]}}"#,
    )
    .unwrap();

    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();

    let settings = std::fs::read_to_string(claude_dir.join("settings.local.json")).unwrap();
    let val: serde_json::Value = serde_json::from_str(&settings).unwrap();
    let allow = val["permissions"]["allow"].as_array().unwrap();
    assert!(allow.contains(&serde_json::json!("Bash(git *)")));
    assert!(allow.iter().any(|v| {
        v.as_str().map(|s| s.contains("agent-chat")).unwrap_or(false)
    }));
}

#[test]
fn init_project_is_idempotent() {
    let tmp = TempDir::new().unwrap();

    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();
    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();

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
fn init_project_creates_claude_md() {
    let tmp = TempDir::new().unwrap();

    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("# Agent Chat"));
    assert!(content.contains("agent-chat read"));
    assert!(content.contains("agent-chat say"));
    assert!(content.contains("agent-chat lock"));
    assert!(content.contains("## Workflow"));
    assert!(content.contains("Starting a task"));
    assert!(content.contains("Don't stop to wait for replies"));
    assert!(content.contains("## Message style"));
}

#[test]
fn init_project_preserves_existing_claude_md() {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join("CLAUDE.md"), "# My Project\n\nDo not delete this.\n").unwrap();

    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();

    let content = std::fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(content.contains("# My Project"));
    assert!(content.contains("Do not delete this."));
    assert!(content.contains("# Agent Chat"));
}

#[test]
fn init_project_does_not_touch_user_files() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init", "--project"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    // No user-level files created
    assert!(!fake_home.path().join(".claude/settings.json").exists());
    assert!(!fake_home.path().join(".claude/CLAUDE.md").exists());
}

// ── --user ──────────────────────────────────────────────────────────

#[test]
fn init_user_creates_user_level_files() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    // Need .git for exclude test
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    cmd()
        .args(["init", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("user"));

    // User-level hooks in ~/.claude/settings.json
    let settings_path = fake_home.path().join(".claude/settings.json");
    assert!(settings_path.exists(), "should create ~/.claude/settings.json");
    let val: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert!(val["hooks"]["SessionStart"].is_array());

    // User-level CLAUDE.md
    let claude_md = fake_home.path().join(".claude/CLAUDE.md");
    assert!(claude_md.exists(), "should create ~/.claude/CLAUDE.md");
    let content = std::fs::read_to_string(&claude_md).unwrap();
    assert!(content.contains("# Agent Chat"));
}

#[test]
fn init_user_does_not_create_project_claude_md() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    // .agent-chat/ created in project
    assert!(tmp.path().join(".agent-chat/log").is_dir());
    // No project-level CLAUDE.md or settings
    assert!(!tmp.path().join("CLAUDE.md").exists());
    assert!(!tmp.path().join(".claude/settings.local.json").exists());
}

#[test]
fn init_user_adds_git_exclude() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    cmd()
        .args(["init", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    let exclude = std::fs::read_to_string(tmp.path().join(".git/info/exclude")).unwrap();
    assert!(exclude.contains(".agent-chat/"));
}

#[test]
fn init_user_git_exclude_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    cmd()
        .args(["init", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();
    cmd()
        .args(["init", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    let exclude = std::fs::read_to_string(tmp.path().join(".git/info/exclude")).unwrap();
    assert_eq!(
        exclude.matches(".agent-chat/").count(),
        1,
        "git exclude entry should not be duplicated"
    );
}

// ── --both ──────────────────────────────────────────────────────────

#[test]
fn init_both_creates_project_and_user_files() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();

    cmd()
        .args(["init", "--both"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("project + user"));

    // Project files
    assert!(tmp.path().join(".agent-chat/log").is_dir());
    assert!(tmp.path().join(".claude/settings.local.json").exists());
    assert!(tmp.path().join("CLAUDE.md").exists());

    // User files
    assert!(fake_home.path().join(".claude/settings.json").exists());
    assert!(fake_home.path().join(".claude/CLAUDE.md").exists());

    // Git exclude
    let exclude = std::fs::read_to_string(tmp.path().join(".git/info/exclude")).unwrap();
    assert!(exclude.contains(".agent-chat/"));
}

#[test]
fn init_both_project_and_user_flags_equals_both() {
    let tmp = TempDir::new().unwrap();
    let fake_home = TempDir::new().unwrap();

    cmd()
        .args(["init", "--project", "--user"])
        .env("HOME", fake_home.path())
        .current_dir(tmp.path())
        .assert()
        .success();

    // Both targets should have files
    assert!(tmp.path().join(".claude/settings.local.json").exists());
    assert!(fake_home.path().join(".claude/settings.json").exists());
}

// ── no flags without stdin shows error ──────────────────────────────

#[test]
fn init_no_flags_no_stdin_shows_error() {
    let tmp = TempDir::new().unwrap();

    // No flags and no stdin → prompt prints, then IO error on stderr
    cmd()
        .args(["init"])
        .current_dir(tmp.path())
        .assert()
        .stderr(predicate::str::contains("Where should hooks"))
        .stderr(predicate::str::contains("no input"));

    // Init should not have completed — no project CLAUDE.md
    assert!(!tmp.path().join("CLAUDE.md").exists());
}
