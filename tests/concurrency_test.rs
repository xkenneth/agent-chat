use assert_cmd::Command;
use assert_fs::TempDir;
use std::thread;

fn cmd() -> Command {
    Command::cargo_bin("agent-chat").unwrap()
}

fn init_project(tmp: &TempDir) {
    cmd().arg("init").current_dir(tmp.path()).assert().success();
}

#[test]
fn concurrent_say_no_corruption() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let dir = tmp.path().to_path_buf();
    let threads: Vec<_> = (0..10)
        .map(|i| {
            let d = dir.clone();
            thread::spawn(move || {
                Command::cargo_bin("agent-chat")
                    .unwrap()
                    .args(["say", &format!("message from thread {}", i)])
                    .current_dir(&d)
                    .env("AGENT_CHAT_NAME", &format!("agent-{}", i))
                    .env("AGENT_CHAT_SESSION_ID", &format!("sess-{}", i))
                    .assert()
                    .success();
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    // All 10 messages should be present
    let log_dir = tmp.path().join(".agent-chat/log");
    let entries: Vec<_> = std::fs::read_dir(&log_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".md") && !name.starts_with(".tmp.")
        })
        .collect();
    assert_eq!(entries.len(), 10, "All 10 messages should be present");

    // Verify no corruption - each file should be parseable
    for entry in &entries {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        assert!(content.starts_with("name: "), "File should start with 'name: '");
        assert!(content.contains("message from thread"), "File should contain message");
    }
}

#[test]
fn concurrent_lock_race() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    let dir = tmp.path().to_path_buf();
    let threads: Vec<_> = (0..2)
        .map(|i| {
            let d = dir.clone();
            thread::spawn(move || {
                let output = Command::cargo_bin("agent-chat")
                    .unwrap()
                    .args(["lock", "src/*.rs"])
                    .current_dir(&d)
                    .env("AGENT_CHAT_NAME", &format!("agent-{}", i))
                    .env("AGENT_CHAT_SESSION_ID", &format!("lock-sess-{}", i))
                    .output()
                    .unwrap();
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stdout.contains("Locked") {
                    "won"
                } else if stderr.contains("Lock conflict") {
                    "lost"
                } else {
                    "unknown"
                }
            })
        })
        .collect();

    let results: Vec<&str> = threads.into_iter().map(|t| t.join().unwrap()).collect();

    // At least one should succeed
    let won = results.iter().filter(|r| **r == "won").count();
    assert!(won >= 1, "At least one agent should acquire the lock");

    // The important guarantee: the final lock file has exactly one owner
    let locks_dir = tmp.path().join(".agent-chat/locks");
    let lock_files: Vec<_> = std::fs::read_dir(&locks_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".lock"))
        .collect();
    assert_eq!(lock_files.len(), 1, "Should have exactly one lock file");
}

#[test]
fn concurrent_read_write_no_panic() {
    let tmp = TempDir::new().unwrap();
    init_project(&tmp);

    // Pre-write some messages
    for i in 0..5 {
        cmd()
            .args(["say", &format!("pre-msg-{}", i)])
            .current_dir(tmp.path())
            .env("AGENT_CHAT_NAME", "setup")
            .env("AGENT_CHAT_SESSION_ID", "setup-sess")
            .assert()
            .success();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let dir = tmp.path().to_path_buf();

    // Spawn writers and readers concurrently
    let mut threads = Vec::new();
    for i in 0..5 {
        let d = dir.clone();
        threads.push(thread::spawn(move || {
            Command::cargo_bin("agent-chat")
                .unwrap()
                .args(["say", &format!("concurrent-msg-{}", i)])
                .current_dir(&d)
                .env("AGENT_CHAT_NAME", &format!("writer-{}", i))
                .env("AGENT_CHAT_SESSION_ID", &format!("writer-sess-{}", i))
                .assert()
                .success();
        }));
    }
    for i in 0..5 {
        let d = dir.clone();
        threads.push(thread::spawn(move || {
            Command::cargo_bin("agent-chat")
                .unwrap()
                .arg("read")
                .current_dir(&d)
                .env("AGENT_CHAT_NAME", &format!("reader-{}", i))
                .env("AGENT_CHAT_SESSION_ID", &format!("reader-sess-{}", i))
                .assert()
                .success();
        }));
    }

    for t in threads {
        t.join().unwrap();
    }
}
