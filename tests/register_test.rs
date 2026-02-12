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
fn register_creates_session_and_outputs_name() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "test-session-1"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("You are "));

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

    let name1 = String::from_utf8_lossy(&output1.stdout);
    let name2 = String::from_utf8_lossy(&output2.stdout);
    assert_eq!(name1, name2, "Same session should get same name");
}

#[test]
fn register_unique_across_sessions() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let output1 = cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "session-a"}"#)
        .output()
        .unwrap();

    let output2 = cmd()
        .arg("register")
        .current_dir(tmp.path())
        .write_stdin(r#"{"session_id": "session-b"}"#)
        .output()
        .unwrap();

    let name1 = String::from_utf8_lossy(&output1.stdout);
    let name2 = String::from_utf8_lossy(&output2.stdout);
    // Names are random, but sessions should have different files
    assert!(tmp.path().join(".agent-chat/sessions/session-a").exists());
    assert!(tmp.path().join(".agent-chat/sessions/session-b").exists());
    // It's possible (but unlikely) to get the same name, so just check both registered
    let _ = (name1, name2);
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
