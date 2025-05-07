mod common;
use common::*;
use predicates::function::function;
use predicates::prelude::*;

#[test]
fn test_pretty_print_default() {
    let test_file = TestFile::new(r#"{"name": "Alice", "age": "30"}"#);
    JqCommand::new()
        .arg(".")
        .arg(test_file.path())
        .assert()
        .success()
        .stdout(json_matches(&[("name", "Alice"), ("age", "30")]));
}

#[test]
fn test_compact_output() {
    let test_file = TestFile::new(r#"{"name": "Alice", "age": "30"}"#);
    JqCommand::new()
        .args(&["-c", ".", test_file.path()])
        .assert()
        .success()
        .stdout(function(|output: &str| {
            let output = output.trim();
            output.contains(r#""name":"Alice"#)
                && output.contains(r#""age":"30"#)
                && output.starts_with("{")
                && output.ends_with("}")
        }));
}

#[test]
fn test_tab_indentation() {
    let test_file = TestFile::new(r#"{"name": "Alice", "age": "30"}"#);
    JqCommand::new()
        .args(&["--tab", ".", test_file.path()])
        .assert()
        .success()
        .stdout(predicate::str::contains("\t"));
}

#[test]
fn test_compact_output_and_tab_conflict() {
    let test_file = TestFile::new(r#"{"name": "Alice", "age": "30"}"#);
    JqCommand::new()
        .args(&["-c", "--tab", ".", test_file.path()])
        .assert()
        .success()
        .stdout(function(|output: &str| {
            let output = output.trim();
            output.contains(r#""name":"Alice"#)
                && output.contains(r#""age":"30"#)
                && output.starts_with("{")
                && output.ends_with("}")
        }));
}

#[test]
fn test_help_output() {
    JqCommand::new()
        .arg("--help")
        .assert()
        .success()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("-c"))
        .stderr(predicate::str::contains("--tab"));
}

#[test]
fn test_version_output() {
    JqCommand::new()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::eq(format!(
            "{} {}\n",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        )));
}

#[test]
fn test_invalid_option() {
    let test_file = TestFile::new(r#"{"name": "test", "value": "123"}"#);
    JqCommand::new()
        .arg("--invalid-option")
        .arg(".")
        .arg(test_file.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown option"));
}

#[test]
fn test_large_json_file() {
    let large_json = r#"{
        "users": [
            {"id": 1, "name": "Alice", "details": {"age": 30, "city": "New York", "hobbies": ["reading", "hiking"]}},
            {"id": 2, "name": "Bob", "details": {"age": 25, "city": "London", "hobbies": ["gaming", "cooking"]}},
            {"id": 3, "name": "Charlie", "details": {"age": 35, "city": "Paris", "hobbies": ["painting", "music"]}}
        ],
        "metadata": {
            "total_users": 3,
            "last_updated": "2024-03-20",
            "settings": {
                "visibility": "public",
                "permissions": ["read", "write"],
                "flags": {
                    "active": true,
                    "verified": true
                }
            }
        }
    }"#;

    let test_file = TestFile::new(large_json);
    JqCommand::new()
        .arg(".")
        .arg(test_file.path())
        .assert()
        .success()
        .stdout(function(|output: &str| {
            output.contains(r#""users""#)
                && output.contains(r#""metadata""#)
                && output.contains(r#""Alice""#)
                && output.contains(r#""total_users": 3"#)
                && output.contains(r#""permissions""#)
                && output.contains(r#""flags""#)
        }));
}

#[test]
fn test_combined_short_options() {
    let test_file = TestFile::new(r#"{"name": "Alice", "age": "30"}"#);
    JqCommand::new()
        .args(&["-cS", ".", test_file.path()])
        .assert()
        .success()
        .stdout(function(|output: &str| {
            let output = output.trim();
            // Check if output is compact (no whitespace between elements)
            output.contains(r#""name":"Alice"#) &&
            output.contains(r#""age":"30"#) &&
            // Check if keys are sorted (age comes before name)
            output == r#"{"age":"30","name":"Alice"}"#
        }));
}

#[test]
fn test_combined_short_options_reversed() {
    let test_file = TestFile::new(r#"{"name": "Alice", "age": "30"}"#);
    JqCommand::new()
        .args(&["-Sc", ".", test_file.path()])
        .assert()
        .success()
        .stdout(function(|output: &str| {
            let output = output.trim();
            // Check if output is compact and sorted, order of flags shouldn't matter
            output == r#"{"age":"30","name":"Alice"}"#
        }));
}

#[test]
fn test_invalid_combined_option() {
    let test_file = TestFile::new(r#"{"name": "test", "value": "123"}"#);
    JqCommand::new()
        .args(&["-cX", ".", test_file.path()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown option: -X"));
}

#[test]
fn test_field_contains_escape_characters() {
    let test_file = TestFile::new(
        r#"{
         "stdout": "rustc 1.85.0 (4d91de4e4 2025-02-17)\nbinary: rustc\ncommit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c4668\n"
       }"#,
    );
    JqCommand::new()
        .args(&["-c", ".", test_file.path()])
        .assert()
        .success()
        .stdout(function(|output: &str| {
            let output = output.trim();
            println!("output: {}", output);
            output == r#"{"stdout":"rustc 1.85.0 (4d91de4e4 2025-02-17)\nbinary: rustc\ncommit-hash: 4d91de4e48198da2e33413efdcd9cd2cc0c4668\n"}"#
        }));
}
