//! Testes de integração E2E para o subcomando `context7 health`.
//!
//! All tests invoke the compiled binary via `assert_cmd::Command::cargo_bin`.
//! No test does real network I/O — the exit 66 and 74 scenarios trigger before
//! do probe HTTP. O isolamento de keys é garantido por `CONTEXT7_HOME` apontando
//! para um `TempDir` vazio.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

// ─── Helper ───

#[allow(deprecated)]
fn cmd_isolado(xdg_home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("context7").unwrap();
    cmd.env_clear()
        .env("CONTEXT7_HOME", xdg_home.path())
        .env("HOME", xdg_home.path());
    cmd
}

// ─── Tests ───

/// `context7 health` without any configured key must exit with code 66.
#[test]
#[serial]
fn test_health_without_keys_returns_66() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir).arg("health").assert().code(66);
}

/// `context7 --json health` without keys must emit parseable JSON and exit with 66.
///
/// Verifies that the NDJSON envelope contains the required fields: `type`, `timestamp`,
/// `config_ok`, `keys_count`, `api_reachable`.
#[test]
#[serial]
fn test_health_json_format_parseable() {
    let dir = TempDir::new().unwrap();
    let output = cmd_isolado(&dir)
        .args(["--json", "health"])
        .output()
        .unwrap();

    // exit 66 esperado (sem keys)
    assert_eq!(output.status.code(), Some(66));

    // stdout deve ser JSON parseável com campos obrigatórios
    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines().next().unwrap_or("");
    let value: serde_json::Value =
        serde_json::from_str(line).expect("stdout deve ser JSON válido em mode --json");

    assert_eq!(value["type"], "health", "campo 'type' deve ser 'health'");
    assert!(
        value["timestamp"].is_string(),
        "campo 'timestamp' deve existir"
    );
    assert_eq!(
        value["config_ok"], true,
        "config_ok deve ser true (client HTTP criado)"
    );
    assert_eq!(value["keys_count"], 0, "keys_count deve ser 0 sem keys");
    assert_eq!(
        value["api_reachable"], false,
        "api_reachable deve ser false sem keys"
    );
}

/// `context7 health --quiet` without keys must exit with 66 and produce empty stdout.
///
/// Ensures that the `--quiet` gate works together with the health subcommand.
#[test]
#[serial]
fn test_health_quiet_silences_stdout() {
    let dir = TempDir::new().unwrap();
    let output = cmd_isolado(&dir)
        .args(["--quiet", "health"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(66));
    assert!(
        output.stdout.is_empty(),
        "stdout deve estar vazio com --quiet, obteve: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}

/// `context7 health` without keys displays a readable message about missing keys.
///
/// Verifies that the human output mentions something about "keys" or the failure symbol.
#[test]
#[serial]
fn test_health_shows_diagnostic_without_keys() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir).arg("health").assert().code(66).stdout(
        predicate::str::contains("keys")
            .or(predicate::str::contains("keys"))
            .or(predicate::str::contains("FAIL"))
            .or(predicate::str::contains("[FAIL]")),
    );
}
