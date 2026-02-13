use assert_cmd::Command;
use assert_fs::TempDir;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

fn init_project(tmp: &TempDir) {
    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();
}

/// Helper to extract additionalContext from JSON output
fn extract_context(stdout: &[u8]) -> String {
    let output_str = String::from_utf8_lossy(stdout);
    let v: serde_json::Value = serde_json::from_str(&output_str)
        .unwrap_or_else(|e| panic!("Failed to parse JSON: {}\nOutput was: {}", e, output_str));
    v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap_or_else(|| panic!("Missing additionalContext in: {}", output_str))
        .to_string()
}

#[test]
fn register_creates_session_and_outputs_name() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let output = cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "test-session-1"}"#)
        .output()
        .unwrap();

    assert!(output.status.success());
    let context = extract_context(&output.stdout);
    assert!(context.contains("You are "), "context should contain identity: {}", context);

    // Session file should exist
    assert!(tmp.path().join(".agent-chat/sessions/test-session-1").exists());
}

#[test]
fn register_is_idempotent_same_session() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let output1 = cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "test-session-2"}"#)
        .output()
        .unwrap();

    let output2 = cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "test-session-2"}"#)
        .output()
        .unwrap();

    let ctx1 = extract_context(&output1.stdout);
    let ctx2 = extract_context(&output2.stdout);

    // Extract name from "You are <name>."
    let name1 = ctx1.split("You are ").nth(1).unwrap().split('.').next().unwrap();
    let name2 = ctx2.split("You are ").nth(1).unwrap().split('.').next().unwrap();
    assert_eq!(name1, name2, "Same session should get same name");
}

#[test]
fn register_unique_across_sessions() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "session-a"}"#)
        .output()
        .unwrap();

    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "session-b"}"#)
        .output()
        .unwrap();

    // Names are random, but sessions should have different files
    assert!(tmp.path().join(".agent-chat/sessions/session-a").exists());
    assert!(tmp.path().join(".agent-chat/sessions/session-b").exists());
}

#[test]
fn register_writes_claude_env_file() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let env_file = tmp.path().join("env_output");

    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .env("CLAUDE_ENV_FILE", env_file.to_str().unwrap())
        .write_stdin(r#"{"session_id": "env-test"}"#)
        .assert()
        .success();

    let content = std::fs::read_to_string(&env_file).unwrap();
    assert!(content.contains("export AGENT_CHAT_NAME="));
    assert!(content.contains("export AGENT_CHAT_SESSION_ID=env-test"));
}

#[test]
fn register_posts_join_message() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "join-test"}"#)
        .assert()
        .success();

    // Check that a "joined the chat" message exists in the log
    let log_dir = tmp.path().join(".agent-chat/log");
    let entries: Vec<_> = std::fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .collect();

    assert!(!entries.is_empty(), "Should have at least one message in log");

    let mut found_join = false;
    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        if content.contains("joined the chat") {
            found_join = true;
            break;
        }
    }
    assert!(found_join, "Should find a 'joined the chat' message");
}

#[test]
fn register_no_duplicate_join_on_resume() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // First registration — should post join message
    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "resume-test"}"#)
        .assert()
        .success();

    // Small delay to avoid timestamp collision
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Second registration (same session_id = resume) — should NOT post join
    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "resume-test"}"#)
        .assert()
        .success();

    // Count "joined the chat" messages
    let log_dir = tmp.path().join(".agent-chat/log");
    let join_count = std::fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".md"))
        .filter(|e| {
            std::fs::read_to_string(e.path())
                .map(|c| c.contains("joined the chat"))
                .unwrap_or(false)
        })
        .count();

    assert_eq!(join_count, 1, "Should have exactly 1 join message, not {}", join_count);
}

#[test]
fn register_injects_existing_messages() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Register agent A first
    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "agent-a-session"}"#)
        .assert()
        .success();

    // Agent A posts a message
    std::thread::sleep(std::time::Duration::from_millis(10));
    cmd()
        .args(["say", "hello from agent A"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "agent-a-name")
        .assert()
        .success();

    std::thread::sleep(std::time::Duration::from_millis(10));

    // Register agent B — should see agent A's messages in additionalContext
    let output = cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "agent-b-session"}"#)
        .output()
        .unwrap();

    assert!(output.status.success());
    let context = extract_context(&output.stdout);

    // Should contain the message from agent A
    assert!(
        context.contains("hello from agent A"),
        "additionalContext should include agent A's message, got: {}",
        context
    );
    // Should also contain the unread header
    assert!(
        context.contains("[agent-chat:"),
        "additionalContext should include unread header, got: {}",
        context
    );
}

#[test]
fn register_accepts_session_id_flag_without_stdin() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let output = cmd()
        .args(["register", "--session-id", "codex-session-1"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    assert!(output.status.success());
    let context = extract_context(&output.stdout);
    assert!(context.contains("You are "), "context should contain identity: {}", context);
    assert!(tmp.path().join(".agent-chat/sessions/codex-session-1").exists());
}

#[test]
fn register_session_id_flag_takes_precedence_over_stdin() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["register", "--session-id", "flag-session"])
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id":"stdin-session"}"#)
        .assert()
        .success();

    assert!(tmp.path().join(".agent-chat/sessions/flag-session").exists());
    assert!(!tmp.path().join(".agent-chat/sessions/stdin-session").exists());
}

#[test]
fn register_rejects_empty_session_id_flag() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["register", "--session-id", "   "])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stderr(predicates::str::contains("session_id cannot be empty"));
}
