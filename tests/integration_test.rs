mod common;
use common::*;
use predicates::function::function;
use predicates::prelude::*;

#[test]
fn test_basic_operations() {
    let test_file = TestFile::new(r#"{"name": "test", "value": "123"}"#);

    // Test with file input
    JqCommand::new()
        .args(&[".", test_file.path()])
        .assert()
        .success()
        .stdout(json_matches(&[("name", "test"), ("value", "123")]));

    // Test with stdin input
    let input_json = r#"{"name": "test", "value": "123"}"#;
    JqCommand::new()
        .arg(".")
        .stdin(input_json)
        .assert()
        .success()
        .stdout(json_matches(&[("name", "test"), ("value", "123")]));
}

#[test]
fn test_filter_expressions() {
    // Test .name
    JqCommand::new()
        .arg(".name")
        .stdin(r#"{"name": "jq"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("jq"));

    // Test .name with different input
    JqCommand::new()
        .arg(".name")
        .stdin(r#"{"name": "Alice", "age": 30}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"));

    // Test .age
    JqCommand::new()
        .arg(".age")
        .stdin(r#"{"age": "twenty"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("twenty"));

    // Test .person.age
    JqCommand::new()
        .arg(".person.age")
        .stdin(r#"{"person.age": "twenty"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("null"));

    // Test .missing (should return null)
    JqCommand::new()
        .arg(".missing")
        .stdin(r#"{"name": "test"}"#)
        .assert()
        .success()
        .stdout(predicate::str::contains("null"));

    // Test empty object
    JqCommand::new()
        .arg(".missing")
        .stdin("{}")
        .assert()
        .success()
        .stdout(predicate::str::contains("null"));

    // Test invalid filter
    JqCommand::new()
        .arg("invalid")
        .stdin("{}")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error parsing filter expression"));

    // Test invalid JSON
    JqCommand::new()
        .arg(".")
        .stdin("invalid json")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to parse JSON"));
}

#[test]
fn test_pretty_print() {
    let input_json = r#"{"name": "test", "value": "123"}"#;

    // Test default pretty print
    JqCommand::new()
        .arg(".")
        .stdin(input_json)
        .assert()
        .success()
        .stdout(json_matches(&[("name", "test"), ("value", "123")]));

    // Test with -c
    JqCommand::new()
        .args(&["-c", "."])
        .stdin(r#"{"status": "ok"}"#)
        .assert()
        .success()
        .stdout(function(|output: &str| {
            let output = output.trim();
            output.contains(r#""status":"ok"#) && output.starts_with("{") && output.ends_with("}")
        }));

    // Test with file input
    let test_file = TestFile::new(r#"{"source": "file"}"#);
    JqCommand::new()
        .arg(".")
        .arg(test_file.path())
        .assert()
        .success()
        .stdout(json_matches(&[("source", "file")]));

    // Test with --tab
    let test_file = TestFile::new(r#"{"source": "file"}"#);
    JqCommand::new()
        .args(&["--tab", "."])
        .arg(test_file.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\t"));
}
