use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::process::Command as StdCommand;

struct ApprenticeGuard {
    name: String,
}

impl ApprenticeGuard {
    fn new(name: &str) -> Self {
        // Clean up any existing apprentice with this name first
        let _ = StdCommand::new("./target/release/srcrr")
            .args(["kill", name])
            .output();

        Self {
            name: name.to_string(),
        }
    }
}

impl Drop for ApprenticeGuard {
    fn drop(&mut self) {
        // Ensure cleanup happens even if test panics
        let _ = StdCommand::new("./target/release/srcrr")
            .args(["kill", &self.name])
            .output();
    }
}

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "🧙‍♂️ The Sorcerer - Command apprentices to do your bidding",
        ))
        .stdout(predicate::str::contains("summon"))
        .stdout(predicate::str::contains("tell"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("kill"))
        .stdout(predicate::str::contains("overview"))
        .stdout(predicate::str::contains("history"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_help_flag() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("-h");

    cmd.assert().success().stdout(predicate::str::contains(
        "🧙‍♂️ The Sorcerer - Command apprentices to do your bidding",
    ));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("invalid_command");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("error: unrecognized subcommand"));
}

#[test]
#[serial]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📋 Listing apprentices..."));
}

#[test]
#[serial]
fn test_overview_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("overview");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of apprentices..."));
}

#[test]
#[serial]
fn test_overview_with_lines_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["overview", "--lines", "10"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of apprentices..."));
}

#[test]
#[serial]
fn test_overview_with_lines_short_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["overview", "-l", "5"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of apprentices..."));
}

#[test]
fn test_summon_without_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("summon");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_tell_without_args() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("tell");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_tell_with_only_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["tell", "test_apprentice"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_kill_without_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("kill");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[serial]
fn test_tell_nonexistent_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["tell", "nonexistent_apprentice", "hello"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "📜 Sending message to apprentice nonexistent_apprentice...",
        ))
        .stdout(predicate::str::contains("💥 The message failed"))
        .stdout(predicate::str::contains("Spell").not());
}

#[test]
#[serial]
fn test_kill_nonexistent_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["kill", "nonexistent_apprentice"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "💀 Killing apprentice nonexistent_apprentice...",
        ))
        .stdout(predicate::str::contains("⚠️  Kill failed"));
}

#[test]
fn test_overview_invalid_lines_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["overview", "--lines", "invalid"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
#[serial]
fn test_list_empty_output_format() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("list");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain either apprentice names or empty message
    assert!(stdout.contains("📋 Listing apprentices..."));
    assert!(stdout.contains("🧙") || stdout.contains("The realm is empty"));
}

#[test]
#[serial]
fn test_overview_empty_output_format() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("overview");

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Should contain either apprentice info or empty message
    assert!(stdout.contains("📊 Overview of apprentices..."));
    assert!(stdout.contains(" Apprentice: ") || stdout.contains("No apprentices found"));
}

// Test that command outputs contain expected emojis and messaging
#[test]
#[serial]
fn test_command_output_formatting() {
    // Use guard to ensure cleanup even if test panics
    let _guard = ApprenticeGuard::new("test_formatting");

    // Test summon command output format
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "test_formatting"]);

    let output = cmd.output().unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("🌟 Summoning apprentice test_formatting..."));
    // Will either succeed or fail, both have specific formatting
    assert!(
        stdout.contains("✨ Apprentice test_formatting has answered your call!")
            || stdout.contains("💀 The summoning failed")
    );

    // Cleanup handled automatically by ApprenticeGuard
}

#[test]
#[serial]
fn test_output_consistency() {
    // Test that repeated calls to list produce consistent output structure
    let mut cmd1 = Command::cargo_bin("srcrr").unwrap();
    cmd1.arg("list");
    let output1 = cmd1.output().unwrap();

    let mut cmd2 = Command::cargo_bin("srcrr").unwrap();
    cmd2.arg("list");
    let output2 = cmd2.output().unwrap();

    let stdout1 = String::from_utf8(output1.stdout).unwrap();
    let stdout2 = String::from_utf8(output2.stdout).unwrap();

    // Both should have the same header
    assert!(stdout1.contains("📋 Listing apprentices..."));
    assert!(stdout2.contains("📋 Listing apprentices..."));

    // Content should be consistent (same apprentices should appear)
    let lines1: Vec<&str> = stdout1.lines().collect();
    let lines2: Vec<&str> = stdout2.lines().collect();

    // Should have same number of lines (assuming no apprentices are added/removed)
    assert_eq!(lines1.len(), lines2.len());
}

#[test]
fn test_history_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["history", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "View and scroll through chat history with an apprentice",
        ))
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("--lines"));
}

#[test]
fn test_history_nonexistent_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["history", "nonexistent-apprentice"]);

    cmd.assert()
        .success() // Command itself succeeds, but shows error message
        .stdout(predicate::str::contains(
            "📜 Viewing chat history for apprentice nonexistent-apprentice",
        ))
        .stdout(predicate::str::contains(
            "💥 Failed to retrieve chat history",
        ));
}

#[test]
fn test_history_with_lines_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["history", "test-apprentice", "--lines", "5"]);

    // Should fail gracefully for non-existent apprentice
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📜 Viewing chat history"));
}

#[test]
fn test_history_command_validation() {
    // Test without apprentice name - should fail
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("history");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}
