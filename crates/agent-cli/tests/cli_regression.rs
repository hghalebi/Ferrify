use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::Value;
use tempfile::TempDir;

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|error| {
            panic!("failed to create parent directory {parent:?}: {error}")
        });
    }
    fs::write(path, contents)
        .unwrap_or_else(|error| panic!("failed to write file {path:?}: {error}"));
}

fn fixture_repo() -> TempDir {
    let tempdir = TempDir::new()
        .unwrap_or_else(|error| panic!("failed to create temp fixture repo: {error}"));
    let root = tempdir.path();

    write_file(
        &root.join("Cargo.toml"),
        r#"[package]
name = "fixture-app"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4", features = ["derive"] }
"#,
    );
    write_file(
        &root.join("src/main.rs"),
        r#"use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, default_value = "world")]
    name: String,
}

fn main() {
    let cli = Cli::parse();
    println!("hello {}", cli.name);
}
"#,
    );
    write_file(&root.join("AGENTS.md"), "# Fixture policy\n");

    write_file(
        &root.join(".agent/modes/architect.yaml"),
        r#"slug: architect
purpose: understand the repository and define a bounded change plan
allowed_capabilities:
  - ReadWorkspace
  - SwitchMode
approval_rules:
  SwitchMode: Allow
"#,
    );
    write_file(
        &root.join(".agent/modes/implementer.yaml"),
        r#"slug: implementer
purpose: make the smallest justified code change
allowed_capabilities:
  - ReadWorkspace
  - EditWorkspace
  - RunChecks
  - SwitchMode
approval_rules:
  EditWorkspace: Ask
  SwitchMode: Allow
validation_minimums:
  must_run:
    - CargoCheck
patch_budget:
  max_files: 3
  max_changed_lines: 120
  allow_manifest_changes: false
"#,
    );
    write_file(
        &root.join(".agent/modes/reviewer.yaml"),
        r#"slug: reviewer
purpose: critique the proposed patch radius
allowed_capabilities:
  - ReadWorkspace
  - SwitchMode
approval_rules:
  SwitchMode: Allow
"#,
    );
    write_file(
        &root.join(".agent/modes/verifier.yaml"),
        r#"slug: verifier
purpose: collect evidence only
allowed_capabilities:
  - ReadWorkspace
  - RunChecks
approval_rules:
  EditWorkspace: Deny
validation_minimums:
  must_run:
    - CargoFmtCheck
    - CargoCheck
    - CargoClippy
    - TargetedTests
reporting:
  may_claim_fix_without_tests: false
"#,
    );
    write_file(
        &root.join(".agent/approvals/default.yaml"),
        r#"slug: default
approval_rules:
  EditWorkspace: Ask
  RunArbitraryCommand: AskIfRisky
  DeleteFiles: AskIfRisky
  NetworkAccess: Deny
forbidden_paths:
  - .git
  - target
dependency_policy: AllowApproved
reporting:
  may_claim_fix_without_tests: false
"#,
    );

    tempdir
}

fn binary_path() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ferrify"))
}

fn run_cli(args: &[&str], root: &Path) -> std::process::Output {
    Command::new(binary_path())
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap_or_else(|error| panic!("failed to execute ferrify: {error}"))
}

fn stdout_json(output: &std::process::Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("stdout should contain valid JSON: {error}"))
}

fn stderr_text(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone())
        .unwrap_or_else(|error| panic!("stderr should be valid UTF-8: {error}"))
}

#[test]
fn scaffold_json_run_verifies_fixture_repo() {
    let fixture = fixture_repo();
    let output = run_cli(
        &[
            "--goal",
            "scaffold fixture repo",
            "--task-kind",
            "scaffold",
            "--auto-approve",
            "--json",
        ],
        fixture.path(),
    );

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr_text(&output)
    );

    let json = stdout_json(&output);
    assert_eq!(json["final_report"]["outcome"]["status"], "Verified");
    assert_eq!(json["change_plan"]["selected_mode"], "architect");
    assert_eq!(
        json["patch_plan"]["target_files"][0],
        Value::String(".agent/modes/implementer.yaml".to_owned())
    );
}

#[test]
fn adversarial_eval_stays_passed() {
    let fixture = fixture_repo();
    let output = run_cli(&["--run-adversarial-policy-eval", "--json"], fixture.path());

    assert!(
        output.status.success(),
        "command failed: {}",
        stderr_text(&output)
    );

    let json = stdout_json(&output);
    assert_eq!(json["name"], "adversarial_policy_eval");
    assert_eq!(json["passed"], Value::Bool(true));
}

#[test]
fn cli_enhancement_without_edit_approval_is_denied() {
    let fixture = fixture_repo();
    let output = run_cli(
        &[
            "--goal",
            "tighten cli",
            "--task-kind",
            "cli-enhancement",
            "--in-scope",
            "src/main.rs",
        ],
        fixture.path(),
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(stderr_text(&output).contains("requires approval"));
}

#[test]
fn invalid_scope_path_is_rejected_at_boundary() {
    let fixture = fixture_repo();
    let output = run_cli(
        &[
            "--goal",
            "reject escaping path",
            "--task-kind",
            "cli-enhancement",
            "--in-scope",
            "../outside",
            "--auto-approve",
        ],
        fixture.path(),
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(stderr_text(&output).contains("must not contain parent-directory traversal"));
}

#[test]
fn invalid_approval_profile_slug_is_rejected_at_cli_boundary() {
    let fixture = fixture_repo();
    let output = run_cli(
        &[
            "--goal",
            "reject invalid profile",
            "--task-kind",
            "scaffold",
            "--approval-profile",
            "Default",
        ],
        fixture.path(),
    );

    assert!(!output.status.success(), "command unexpectedly succeeded");
    assert!(stderr_text(&output).contains("invalid CLI input"));
}
