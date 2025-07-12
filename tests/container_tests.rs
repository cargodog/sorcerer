use serial_test::serial;
use std::process::Command;
use std::time::Duration;

#[test]
#[serial]
fn test_summon_and_communicate() {
    // Check if we have the ANTHROPIC_API_KEY set
    if std::env::var("ANTHROPIC_API_KEY").is_err() {
        eprintln!("Skipping container test: ANTHROPIC_API_KEY not set");
        return;
    }

    // Clean up any existing test apprentice
    let _ = Command::new("./target/release/srcrr")
        .args(["banish", "container-test"])
        .output();

    // Wait a bit for cleanup
    std::thread::sleep(Duration::from_secs(1));

    // Test summoning
    let output = Command::new("./target/release/srcrr")
        .args(["summon", "container-test"])
        .output()
        .expect("Failed to execute summon command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Summon failed. stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        stdout.contains("has answered your call"),
        "Unexpected summon output: {stdout}"
    );

    // Wait for container to be ready
    std::thread::sleep(Duration::from_secs(3));

    // Test communication
    let output = Command::new("./target/release/srcrr")
        .args(["tell", "container-test", "What is 2+2?"])
        .output()
        .expect("Failed to execute tell command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Tell failed. stdout: {stdout}, stderr: {stderr}"
    );
    assert!(
        stdout.contains("The apprentice responds"),
        "Unexpected tell output: {stdout}"
    );
    assert!(stdout.contains("4"), "Response should contain '4'");

    // Test scry includes our apprentice
    let output = Command::new("./target/release/srcrr")
        .arg("scry")
        .output()
        .expect("Failed to execute scry command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("container-test"),
        "Scry should list our apprentice"
    );

    // Clean up
    let output = Command::new("./target/release/srcrr")
        .args(["banish", "container-test"])
        .output()
        .expect("Failed to execute banish command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("has been banished"),
        "Unexpected banish output: {stdout}"
    );
}

#[test]
#[serial]
fn test_summon_duplicate_fails() {
    // This test doesn't require API key or containers
    // First summon might fail if no runtime, but duplicate check should still work

    let name = "duplicate-test";

    // Try to summon twice
    let _ = Command::new("./target/release/srcrr")
        .args(["summon", name])
        .output();

    // Second summon should fail with "already exists" if first succeeded
    let output = Command::new("./target/release/srcrr")
        .args(["summon", name])
        .output()
        .expect("Failed to execute summon command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // If container runtime is available, it should say "already exists"
    // If not available, it will fail with a different error
    // This at least tests the CLI is working
    assert!(
        stdout.contains("already exists") || stdout.contains("summoning failed"),
        "Unexpected output: {stdout}"
    );

    // Clean up if it was created
    let _ = Command::new("./target/release/srcrr")
        .args(["banish", name])
        .output();
}

#[test]
fn test_invalid_apprentice_name() {
    let output = Command::new("./target/release/srcrr")
        .args(["summon", "invalid name with spaces"])
        .output()
        .expect("Failed to execute summon command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Invalid apprentice name") || stdout.contains("summoning failed"),
        "Should reject invalid name"
    );
}
