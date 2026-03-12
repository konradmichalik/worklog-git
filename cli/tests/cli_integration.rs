use std::process::Command;

fn cargo_run(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--package", "devcap-cli", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to execute cargo run")
}

#[test]
fn help_flag_shows_usage() {
    let output = cargo_run(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Aggregate git commits"));
}

#[test]
fn json_flag_produces_valid_json() {
    let output = cargo_run(&["--json", "--path", "/tmp"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let trimmed = stdout.trim();
    assert!(
        serde_json::from_str::<serde_json::Value>(trimmed).is_ok(),
        "Output was not valid JSON: {trimmed}"
    );
}

#[test]
fn invalid_period_shows_error() {
    let output = cargo_run(&["-p", "foobar"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success());
    assert!(
        stderr.contains("Unknown period") || stderr.contains("invalid value"),
        "Unexpected error: {stderr}"
    );
}

#[test]
fn since_flag_accepted() {
    let output = cargo_run(&["--since", "2026-03-01", "--path", "/tmp"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn until_flag_accepted() {
    let output = cargo_run(&["--until", "2030-12-31", "--path", "/tmp"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn since_and_until_together_accepted() {
    let output = cargo_run(&[
        "--since",
        "2026-03-01",
        "--until",
        "2026-03-10",
        "--path",
        "/tmp",
    ]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn invalid_since_date_shows_error() {
    let output = cargo_run(&["--since", "not-a-date"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("invalid value"),
        "Unexpected error: {stderr}"
    );
}

#[test]
fn inverted_date_range_shows_error() {
    let output = cargo_run(&["--since", "2026-03-10", "--until", "2026-03-01"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("must be on or before"),
        "Unexpected error: {stderr}"
    );
}

#[test]
fn nonexistent_path_shows_message() {
    let output = cargo_run(&["--path", "/tmp/nonexistent_devcap_test_dir"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No git repositories found") || output.status.success(),
        "Unexpected output: {stderr}"
    );
}
