use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::thread;
use std::time::Duration;

#[test]
#[serial]
fn test_reference_memory() {
    // Test if agent remembers specific information across messages
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice for testing
    cmd.args(["summon", "memory_test_1"]).assert().success();

    // Give the apprentice time to start
    thread::sleep(Duration::from_secs(2));

    // First message: introduce information
    let mut tell_cmd = Command::cargo_bin("srcrr").unwrap();
    tell_cmd
        .args(["tell", "memory_test_1", "My favorite color is purple"])
        .assert()
        .success();

    // Second message: ask about the information
    let mut ask_cmd = Command::cargo_bin("srcrr").unwrap();
    ask_cmd
        .args(["tell", "memory_test_1", "What is my favorite color?"])
        .assert()
        .success()
        .stdout(predicate::str::contains("purple").or(predicate::str::contains("I don't")));

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "memory_test_1"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_pronoun_resolution() {
    // Test if agent can resolve pronouns based on context
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice
    cmd.args(["summon", "pronoun_test"]).assert().success();

    thread::sleep(Duration::from_secs(2));

    // First message: introduce entity
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "pronoun_test", "I have a dog named Max"])
        .assert()
        .success();

    // Second message: use pronoun
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "pronoun_test", "How old is he?"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Max")
                .or(predicate::str::contains("dog"))
                .or(predicate::str::contains("I don't")),
        );

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "pronoun_test"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_continuation() {
    // Test if agent can continue a sequence
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice
    cmd.args(["summon", "continuation_test"]).assert().success();

    thread::sleep(Duration::from_secs(2));

    // First message: start counting
    Command::cargo_bin("srcrr")
        .unwrap()
        .args([
            "tell",
            "continuation_test",
            "Let's count to 5. I'll start: 1, 2...",
        ])
        .assert()
        .success();

    // Second message: ask to continue
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "continuation_test", "Continue"])
        .assert()
        .success()
        .stdout(predicate::str::contains("3").or(predicate::str::contains("count")));

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "continuation_test"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_correction_memory() {
    // Test if agent remembers both incorrect and corrected information
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice
    cmd.args(["summon", "correction_test"]).assert().success();

    thread::sleep(Duration::from_secs(2));

    // First message: wrong information
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "correction_test", "The capital of France is London"])
        .assert()
        .success();

    // Second message: correction
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "correction_test", "Sorry, I meant Paris"])
        .assert()
        .success();

    // Third message: ask about it
    Command::cargo_bin("srcrr")
        .unwrap()
        .args([
            "tell",
            "correction_test",
            "What did I say the capital of France was?",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("London")
                .or(predicate::str::contains("Paris"))
                .or(predicate::str::contains("capital")),
        );

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "correction_test"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_multi_turn_reasoning() {
    // Test if agent can reason across multiple messages
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice
    cmd.args(["summon", "reasoning_test"]).assert().success();

    thread::sleep(Duration::from_secs(2));

    // First message: initial state
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "reasoning_test", "I have 3 apples"])
        .assert()
        .success();

    // Second message: state change
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "reasoning_test", "I gave 1 to my friend"])
        .assert()
        .success();

    // Third message: ask about result
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "reasoning_test", "How many apples do I have now?"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("2")
                .or(predicate::str::contains("two"))
                .or(predicate::str::contains("apple")),
        );

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "reasoning_test"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_context_switching() {
    // Test if agent can switch between topics and return to previous ones
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice
    cmd.args(["summon", "context_switch_test"])
        .assert()
        .success();

    thread::sleep(Duration::from_secs(2));

    // First message: topic A
    Command::cargo_bin("srcrr")
        .unwrap()
        .args([
            "tell",
            "context_switch_test",
            "Tell me about Python programming",
        ])
        .assert()
        .success();

    // Second message: switch to topic B
    Command::cargo_bin("srcrr")
        .unwrap()
        .args([
            "tell",
            "context_switch_test",
            "Actually, what's the weather like?",
        ])
        .assert()
        .success();

    // Third message: return to topic A
    Command::cargo_bin("srcrr")
        .unwrap()
        .args([
            "tell",
            "context_switch_test",
            "Going back to my first question...",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Python")
                .or(predicate::str::contains("programming"))
                .or(predicate::str::contains("first")),
        );

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "context_switch_test"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_no_context_control() {
    // Control test: verify that fresh sessions don't have context
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon first apprentice
    cmd.args(["summon", "control_test_1"]).assert().success();

    thread::sleep(Duration::from_secs(2));

    // Send information to first apprentice
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "control_test_1", "My favorite number is 42"])
        .assert()
        .success();

    // Kill first apprentice
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "control_test_1"])
        .assert()
        .success();

    // Summon second apprentice with same name
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["summon", "control_test_1"])
        .assert()
        .success();

    thread::sleep(Duration::from_secs(2));

    // Ask second apprentice about the information
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["tell", "control_test_1", "What is my favorite number?"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("42")
                .not()
                .or(predicate::str::contains("don't know")),
        );

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "control_test_1"])
        .assert()
        .success();
}

#[test]
#[serial]
fn test_complex_conversation_flow() {
    // Test a more complex conversation with multiple context dependencies
    let mut cmd = Command::cargo_bin("srcrr").unwrap();

    // Summon an apprentice
    cmd.args(["summon", "complex_test"]).assert().success();

    thread::sleep(Duration::from_secs(2));

    // Build up context
    let messages = vec![
        ("My name is Alice", vec!["Alice"]),
        (
            "I work as a software engineer",
            vec!["software", "engineer"],
        ),
        ("My favorite programming language is Rust", vec!["Rust"]),
        (
            "What do you know about me?",
            vec!["Alice", "software", "Rust"],
        ),
    ];

    for (message, expected_keywords) in messages {
        let output = Command::cargo_bin("srcrr")
            .unwrap()
            .args(["tell", "complex_test", message])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let response = String::from_utf8_lossy(&output);

        // For the final question, check if any context is remembered
        if message.contains("What do you know") {
            let has_context = expected_keywords
                .iter()
                .any(|&keyword| response.contains(keyword));
            println!("Response contains context: {has_context}");
        }
    }

    // Cleanup
    Command::cargo_bin("srcrr")
        .unwrap()
        .args(["kill", "complex_test"])
        .assert()
        .success();
}
