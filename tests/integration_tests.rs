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
            .args(["rm", name])
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
            .args(["rm", &self.name])
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
        .stdout(predicate::str::contains("ls"))
        .stdout(predicate::str::contains("rm"))
        .stdout(predicate::str::contains("ps"))
        .stdout(predicate::str::contains("show"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("srcrr"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_summon_missing_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("summon");

    // With Vec<String>, empty args is valid, should show error message
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("❌ No apprentice names provided"));
}

#[test]
fn test_tell_missing_args() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("tell");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
#[serial]
fn test_ls_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📋 Listing apprentices..."));
}

#[test]
#[serial]
fn test_ps_command() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ps");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of apprentices..."));
}

#[test]
#[serial]
fn test_ps_with_lines_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--lines", "10"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Overview of apprentices..."));
}

#[test]
fn test_ps_invalid_lines_value() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--lines", "not-a-number"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_rm_missing_name() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("rm");

    // With Vec<String>, empty args is valid, should show error message
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("❌ No apprentice names provided"));
}

#[test]
fn test_rm_nonexistent_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "nonexistent-apprentice"]);

    cmd.assert()
        .success() // Command itself succeeds, but shows error message
        .stdout(predicate::str::contains("💀 Removing apprentice"))
        .stdout(predicate::str::contains("⚠️  Failed to remove"));
}

#[test]
#[serial]
#[ignore] // Requires working container runtime
fn test_summon_and_tell_flow() {
    let _guard = ApprenticeGuard::new("test-flow");

    // Summon an apprentice
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "test-flow"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "🌟 Summoning apprentice test-flow",
        ))
        .stdout(predicate::str::contains(
            "✨ Apprentice test-flow has answered",
        ));

    // Send a message
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["tell", "test-flow", "What is 2+2?"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📜 Sending message"))
        .stdout(predicate::str::contains("🔮 The apprentice responds"));

    // List should include our apprentice
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-flow"));

    // Remove the apprentice
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "test-flow"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("💀 Removing apprentice test-flow"))
        .stdout(predicate::str::contains(
            "⚰️  Apprentice test-flow has been removed",
        ));
}

#[test]
fn test_summon_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Create and start new apprentice containers",
        ))
        .stdout(predicate::str::contains("NAMES"));
}

#[test]
fn test_tell_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["tell", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Send a message to an apprentice and get its response",
        ))
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("MESSAGE"));
}

#[test]
fn test_ls_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ls", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List all active apprentices"));
}

#[test]
fn test_rm_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Stop and remove apprentice containers",
        ))
        .stdout(predicate::str::contains("NAMES"));
}

#[test]
fn test_ps_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Show detailed status information for all apprentices",
        ))
        .stdout(predicate::str::contains("--lines"));
}

#[test]
fn test_tell_nonexistent_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["tell", "nonexistent-apprentice", "Hello"]);

    cmd.assert()
        .success() // Command itself succeeds, but shows error message
        .stdout(predicate::str::contains(
            "📜 Sending message to apprentice nonexistent-apprentice",
        ))
        .stdout(predicate::str::contains("💥 The message failed"));
}

#[test]
fn test_tell_invalid_args() {
    // Test with only name, no message
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["tell", "test-apprentice"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_multiple_subcommands_error() {
    // Test that we can't use multiple subcommands
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "ls"]);

    // This should treat "ls" as an apprentice name, not a command
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🌟 Summoning apprentice ls"));
}

#[test]
fn test_ps_with_negative_lines() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["ps", "--lines", "-5"]);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("unexpected argument"));
}

#[test]
fn test_empty_ls_output() {
    // Ensure all apprentices are removed first
    let _ = StdCommand::new("./target/release/srcrr")
        .args(["rm", "-a"])
        .output();

    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📋 Listing apprentices"))
        .stdout(predicate::str::contains(
            "The realm is empty - no apprentices found",
        ));
}

#[test]
fn test_show_command_help() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["show", "--help"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "View and scroll through chat history with an apprentice",
        ))
        .stdout(predicate::str::contains("NAME"))
        .stdout(predicate::str::contains("--lines"));
}

#[test]
fn test_show_nonexistent_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["show", "nonexistent-apprentice"]);

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
fn test_show_with_lines_option() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["show", "test-apprentice", "--lines", "5"]);

    // Should fail gracefully for non-existent apprentice
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📜 Viewing chat history"));
}

#[test]
fn test_show_command_validation() {
    // Test without apprentice name - should fail
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("show");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

// New tests for multiple apprentice functionality

#[test]
fn test_summon_multiple_apprentices() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "alice", "bob", "carol"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🌟 Summoning apprentice alice"))
        .stdout(predicate::str::contains("🌟 Summoning apprentice bob"))
        .stdout(predicate::str::contains("🌟 Summoning apprentice carol"))
        .stdout(predicate::str::contains("📊 Summary:"));
}

#[test]
fn test_rm_multiple_apprentices() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "alice", "bob"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("💀 Removing apprentice alice"))
        .stdout(predicate::str::contains("💀 Removing apprentice bob"))
        .stdout(predicate::str::contains("📊 Summary:"));
}

#[test]
fn test_rm_all_flag() {
    // First ensure we have no apprentices
    let _ = StdCommand::new("./target/release/srcrr")
        .args(["rm", "-a"])
        .output();

    // Test with no apprentices
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "-a"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📭 No apprentices to remove"));
}

#[test]
fn test_rm_all_flag_long_form() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "--all"]);

    cmd.assert().success().stdout(
        predicate::str::contains("No apprentices to remove")
            .or(predicate::str::contains("Removing all")),
    );
}

#[test]
fn test_rm_with_all_flag_and_names_conflict() {
    // The -a flag should take precedence over individual names
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "-a", "alice", "bob"]);

    // This should remove all apprentices, ignoring the individual names
    cmd.assert().success().stdout(
        predicate::str::contains("No apprentices to remove")
            .or(predicate::str::contains("Removing all")),
    );
}

#[test]
fn test_summon_single_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "single-test"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "🌟 Summoning apprentice single-test",
        ))
        // Should NOT show summary for single apprentice
        .stdout(predicate::str::contains("📊 Summary:").not());
}

#[test]
fn test_rm_single_apprentice() {
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "single-test"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "💀 Removing apprentice single-test",
        ))
        // Should NOT show summary for single apprentice
        .stdout(predicate::str::contains("📊 Summary:").not());
}

#[test]
#[serial]
#[ignore] // Requires working container runtime
fn test_summon_and_rm_multiple_flow() {
    // Clean up first
    let _ = StdCommand::new("./target/release/srcrr")
        .args(["rm", "-a"])
        .output();

    // Summon multiple apprentices
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["summon", "test-multi-1", "test-multi-2", "test-multi-3"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Summary:"));

    // List should show all three
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-multi-1"))
        .stdout(predicate::str::contains("test-multi-2"))
        .stdout(predicate::str::contains("test-multi-3"));

    // Remove two of them
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "test-multi-1", "test-multi-2"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("📊 Summary: 2/2"));

    // List should only show one
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.arg("ls");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("test-multi-3"))
        .stdout(predicate::str::contains("test-multi-1").not())
        .stdout(predicate::str::contains("test-multi-2").not());

    // Remove all remaining
    let mut cmd = Command::cargo_bin("srcrr").unwrap();
    cmd.args(["rm", "-a"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("🗑️  Removing all 1 apprentices"));
}
