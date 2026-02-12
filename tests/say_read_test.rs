use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

fn init_project(tmp: &TempDir) {
    cmd().args(["init", "--project"]).current_dir(tmp.path()).assert().success();
}

#[test]
fn say_creates_message_file() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .args(["say", "hello", "world"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    let log_dir = tmp.path().join(".agent-chat/log");
    let entries: Vec<_> = std::fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);

    let content = std::fs::read_to_string(entries[0].path()).unwrap();
    assert!(content.contains("name: swift-fox"));
    assert!(content.contains("hello world"));
}

#[test]
fn read_shows_messages() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Post a message
    cmd()
        .args(["say", "test message"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Read from another session
    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::contains("swift-fox"))
        .stdout(predicate::str::contains("test message"));
}

#[test]
fn read_advances_cursor() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Post a message
    cmd()
        .args(["say", "first"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Read once
    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::contains("first"));

    // Wait a bit then read again - should show nothing
    std::thread::sleep(std::time::Duration::from_millis(50));

    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn read_first_session_shows_last_5() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Post 10 messages
    for i in 0..10 {
        cmd()
            .args(["say", &format!("msg-{}", i)])
            .current_dir(tmp.path())
            .env("AGENT_CHAT_NAME", "swift-fox")
            .env("AGENT_CHAT_SESSION_ID", "sess1")
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // New session reads - should see last 5
    let output = cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "new-session")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 5);
    assert!(stdout.contains("msg-5"));
    assert!(stdout.contains("msg-9"));
    assert!(!stdout.contains("msg-4"));
}

#[test]
fn read_all_shows_everything() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    for i in 0..3 {
        cmd()
            .args(["say", &format!("msg-{}", i)])
            .current_dir(tmp.path())
            .env("AGENT_CHAT_NAME", "swift-fox")
            .env("AGENT_CHAT_SESSION_ID", "sess1")
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let output = cmd()
        .args(["read", "--all"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 3);
}

#[test]
fn read_skips_own_messages() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Agent posts a message
    cmd()
        .args(["say", "my own message"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // Same agent reads — should see nothing (own message filtered)
    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn read_skips_own_shows_others() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts
    cmd()
        .args(["say", "from A"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    std::thread::sleep(std::time::Duration::from_millis(10));

    // B posts
    cmd()
        .args(["say", "from B"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success();

    // A reads — should see only B's message
    let output = cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bold-hawk"));
    assert!(stdout.contains("from B"));
    assert!(!stdout.contains("from A"));
}

#[test]
fn read_all_skips_own_messages() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts
    cmd()
        .args(["say", "own msg"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    std::thread::sleep(std::time::Duration::from_millis(10));

    // B posts
    cmd()
        .args(["say", "other msg"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success();

    // A reads --all — should see only B's message
    let output = cmd()
        .args(["read", "--all"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("other msg"));
    assert!(!stdout.contains("own msg"));
}

#[test]
fn read_cursor_advances_past_own() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // A posts (own message)
    cmd()
        .args(["say", "own msg"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success();

    // A reads — empty (own message filtered), but cursor should advance
    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    std::thread::sleep(std::time::Duration::from_millis(50));

    // B posts
    cmd()
        .args(["say", "from B"])
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess2")
        .assert()
        .success();

    // A reads again — should see only B's message
    let output = cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "swift-fox")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("from B"));
    assert!(!stdout.contains("own msg"));
}

#[test]
fn read_no_messages_no_output() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    cmd()
        .arg("read")
        .current_dir(tmp.path())
        .env("AGENT_CHAT_NAME", "bold-hawk")
        .env("AGENT_CHAT_SESSION_ID", "sess1")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}
