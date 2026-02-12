use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

fn init_project(tmp: &TempDir) {
    cmd().arg("init").current_dir(tmp.path()).assert().success();
}

#[test]
fn check_lock_warns_on_locked_file() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Lock a pattern
    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Check from different session
    let stdin_json = r#"{"tool_name": "Edit", "tool_input": {"file_path": "src/main.rs", "old_string": "foo", "new_string": "bar"}}"#;

    cmd()
        .arg("check-lock")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .write_stdin(stdin_json)
        .assert()
        .success()
        .stdout(predicate::str::contains("hookSpecificOutput"))
        .stdout(predicate::str::contains("WARNING"))
        .stdout(predicate::str::contains("swift-fox"));
}

#[test]
fn check_lock_silent_on_own_lock() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["lock", "src/*.rs"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Check from same session - should be silent
    let stdin_json = r#"{"tool_name": "Edit", "tool_input": {"file_path": "src/main.rs", "old_string": "foo", "new_string": "bar"}}"#;

    cmd()
        .arg("check-lock")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .write_stdin(stdin_json)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn check_lock_silent_when_unlocked() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let stdin_json = r#"{"tool_name": "Write", "tool_input": {"file_path": "src/new_file.rs", "content": "fn main() {}"}}"#;

    cmd()
        .arg("check-lock")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .write_stdin(stdin_json)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn hook_stdin_session_start_format() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Simulate the actual hook stdin format
    let stdin_json = r#"{"session_id": "hook-test-session", "session_type": "startup"}"#;

    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(stdin_json)
        .assert()
        .success()
        .stdout(predicate::str::contains("You are "));
}

#[test]
fn hook_stdin_stop_format() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Stop hook doesn't read stdin, just checks status
    cmd()
        .arg("status")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_SESSION_ID", "stop-test")
        .assert()
        .success();
}
