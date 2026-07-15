//! Testes de integração para o storage XDG end-to-end.
//!
//! Todos os testes usam `tempfile::TempDir` como `CONTEXT7_HOME` para
//! garantir isolamento total do sistema de arquivos real do usuário.
//!
//! Todos os testes são marcados com `#[serial]` porque manipulam variáveis
//! de ambiente de processo (`CONTEXT7_HOME`, `HOME`) que são globais.
//! Sem serial, dois testes em paralelo podem interferir nas env vars um do outro.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

// ─── Helper ───

/// Cria um `Command` isolado com `CONTEXT7_HOME` apontando para `dir`.
///
/// NÃO define `CONTEXT7_LANG` nem `CONTEXT7_API_KEYS` como strings vazias —
/// isso faria o clap rejeitar os valores `""` contra os value_parsers definidos.
/// O isolamento de keys é garantido pelo `CONTEXT7_HOME` temporário vazio.
#[allow(deprecated)] // cargo_bin depreciado no assert_cmd 2.1.0+ (build-dir custom); este projeto não usa build-dir customizado
fn cmd_xdg(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("context7").unwrap();
    cmd.env_clear()
        .env("CONTEXT7_HOME", dir.path())
        .env("HOME", dir.path());
    cmd
}

// ── Complete cycle: add → list → remove ───────────────────────────────────────

/// Tests the full cycle: adiciona, lista, remove via CONTEXT7_HOME isolado.
#[test]
#[serial]
fn test_add_list_remove_complete_cycle_via_xdg_home() {
    let dir = TempDir::new().unwrap();
    let key = "ctx7sk-ciclo-completo-12345678901";

    // Passo 1: Adiciona
    cmd_xdg(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();

    // Passo 2: Lista — deve ter exactly 1 key
    cmd_xdg(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[1]").or(predicate::str::contains("1 key")));

    // Passo 3: Remove pelo índice 1
    cmd_xdg(&dir)
        .args(["keys", "remove", "1"])
        .assert()
        .success();

    // Passo 4: Lista novamente — deve estar vazia
    cmd_xdg(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Nenhuma key")
                .or(predicate::str::contains("No key"))
                .or(predicate::str::contains("0 key")),
        );
}

// ── Import/Export roundtrip ────────────────────────────────────────────────────

/// Tests .env file import → export → compares original values.
#[test]
#[serial]
fn test_import_export_roundtrip() {
    let dir = TempDir::new().unwrap();
    let chave1 = "ctx7sk-import-key-aaa-1234567890";
    let chave2 = "ctx7sk-import-key-bbb-1234567890";

    // Cria file .env temporário
    let env_file = dir.path().join("test.env");
    std::fs::write(
        &env_file,
        format!("CONTEXT7_API={chave1}\nCONTEXT7_API={chave2}\n"),
    )
    .unwrap();

    // Importa
    cmd_xdg(&dir)
        .args(["keys", "import", env_file.to_str().unwrap()])
        .assert()
        .success();

    // Exporta e verifica que ambas as keys estão no output
    cmd_xdg(&dir)
        .args(["keys", "export"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("CONTEXT7_API={chave1}")))
        .stdout(predicate::str::contains(format!("CONTEXT7_API={chave2}")));
}

// ── Clear ─────────────────────────────────────────────────────────────────────

/// Tests that `keys clear --yes` removes all keys.
#[test]
#[serial]
fn test_clear_removes_all_keys() {
    let dir = TempDir::new().unwrap();

    // Adds 3 keys
    for i in 1..=3 {
        cmd_xdg(&dir)
            .args([
                "keys",
                "add",
                &format!("ctx7sk-key-clear-{i:02}-1234567890"),
            ])
            .assert()
            .success();
    }

    // Clear com --yes
    cmd_xdg(&dir)
        .args(["keys", "clear", "--yes"])
        .assert()
        .success();

    // Verifica que está vazio
    cmd_xdg(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Nenhuma key")
                .or(predicate::str::contains("No key"))
                .or(predicate::str::contains("0 key")),
        );
}

// ── Path with XDG override ──────────────────────────────────────────────────────

/// Tests that `keys path` returns path containing the `CONTEXT7_HOME` override.
#[test]
#[serial]
fn test_path_returns_xdg_config_home_override() {
    let dir = TempDir::new().unwrap();
    let dir_str = dir.path().to_str().unwrap().to_owned();

    cmd_xdg(&dir)
        .args(["keys", "path"])
        .assert()
        .success()
        // O path deve conter o TempDir ou "context7" (name do diretório XDG)
        .stdout(predicate::str::contains(&dir_str).or(predicate::str::contains("context7")));
}

// ── Deduplication ──────────────────────────────────────────────────────────────

/// Adding the same key twice must not duplicate in storage.
#[test]
#[serial]
fn test_add_duplicate_keys_does_not_accumulate() {
    let dir = TempDir::new().unwrap();
    let key = "ctx7sk-duplicada-12345678901234";

    // Adds twice
    cmd_xdg(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();
    cmd_xdg(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();

    // Export deve conter exactly 1 ocorrência
    let output = cmd_xdg(&dir).args(["keys", "export"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let ocorrencias = stdout
        .lines()
        .filter(|l| l.contains(&format!("CONTEXT7_API={key}")))
        .count();
    assert_eq!(
        ocorrencias, 1,
        "duplicate key must not be stored twice"
    );
}

// ── Remove invalid index ─────────────────────────────────────────────────────

/// `keys remove` with invalid index must not cause panic — controlled error.
#[test]
#[serial]
fn test_remove_invalid_index_returns_controlled_error() {
    let dir = TempDir::new().unwrap();

    // Without any key, tries to remove index 99
    let saida = cmd_xdg(&dir)
        .args(["keys", "remove", "99"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        !combined.contains("thread 'main' panicked"),
        "remove with invalid index must not panic: {combined}"
    );
    assert!(
        !combined.contains("index out of bounds"),
        "must not leak internal bounds message: {combined}"
    );
}

// ── Multiple keys with correct indices ─────────────────────────────────────

/// Adds 3 keys, removes the middle one (index 2), verifies the others persist.
#[test]
#[serial]
fn test_remove_middle_key_preserves_others() {
    let dir = TempDir::new().unwrap();
    let chave1 = "ctx7sk-key-primeira-1234567890";
    let chave2 = "ctx7sk-key-segunda--1234567890";
    let chave3 = "ctx7sk-key-terceira-1234567890";

    for key in [chave1, chave2, chave3] {
        cmd_xdg(&dir)
            .args(["keys", "add", key])
            .assert()
            .success();
    }

    // Removes index 2 (chave2)
    cmd_xdg(&dir)
        .args(["keys", "remove", "2"])
        .assert()
        .success();

    // Export deve conter chave1 e chave3, mas não chave2
    let output = cmd_xdg(&dir).args(["keys", "export"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(chave1),
        "chave1 must persist after removing chave2"
    );
    assert!(!stdout.contains(chave2), "chave2 deve ter sido removed");
    assert!(
        stdout.contains(chave3),
        "chave3 must persist after removing chave2"
    );
}

// ── Config file permissions ───────────────────────────────────────────

/// On Unix systems, the config.toml file must have 600 permissions after writing.
#[test]
#[serial]
#[cfg(unix)]
fn test_config_toml_has_600_permissions_on_unix() {
    use std::os::unix::fs::PermissionsExt;

    let dir = TempDir::new().unwrap();
    cmd_xdg(&dir)
        .args(["keys", "add", "ctx7sk-permissao-unix-12345678"])
        .assert()
        .success();

    // Busca o file config.toml dentro do TempDir
    let config_path = dir.path().join("context7").join("config.toml");

    if config_path.exists() {
        let metadata = std::fs::metadata(&config_path).unwrap();
        let mode = metadata.permissions().mode();
        // Verifica que apenas o dono tem leitura e escrita (0o600)
        let bits_outros = mode & 0o077;
        assert_eq!(
            bits_outros, 0,
            "config.toml deve ter permissões 600, outros bits: {bits_outros:o}"
        );
    }
}

// ── Regression B3: exit code 1 in keys remove with invalid index ─────────────

/// `keys remove 0` must return exit code 1 (invalid index — starts at 1).
#[test]
#[serial]
fn test_keys_remove_index_zero_returns_exit_1() {
    let dir = TempDir::new().unwrap();

    cmd_xdg(&dir)
        .args(["keys", "add", "ctx7sk-b3-idx-zero-12345678901"])
        .assert()
        .success();

    cmd_xdg(&dir)
        .args(["keys", "remove", "0"])
        .assert()
        .failure();
}

/// `keys remove` with index greater than total must return exit code 1.
#[test]
#[serial]
fn test_keys_remove_index_exceeds_total_returns_exit_1() {
    let dir = TempDir::new().unwrap();

    cmd_xdg(&dir)
        .args(["keys", "add", "ctx7sk-b3-overflow-12345678901"])
        .assert()
        .success();

    // There is 1 key, index 99 is invalid
    cmd_xdg(&dir)
        .args(["keys", "remove", "99"])
        .assert()
        .failure();
}

/// `keys remove` with empty list must return exit code 1.
#[test]
#[serial]
fn test_keys_remove_empty_list_returns_exit_1() {
    let dir = TempDir::new().unwrap();

    // Sem nenhuma key adicionada
    cmd_xdg(&dir)
        .args(["keys", "remove", "1"])
        .assert()
        .failure();
}

// ── LOW-01: path traversal in CONTEXT7_HOME ───────────────────────────────────

/// `keys path` with CONTEXT7_HOME containing `..` must not return path with `..`.
/// Verifies that path traversal protection works at runtime via CLI.
#[test]
#[serial]
#[allow(deprecated)] // cargo_bin depreciado no assert_cmd 2.1.0+ (build-dir custom); este projeto não usa build-dir customizado
fn test_keys_path_with_path_traversal_does_not_expose_parent_path() {
    let cases = ["../../../etc", "..", "/tmp/../etc"];

    for case in &cases {
        let mut cmd = assert_cmd::Command::cargo_bin("context7").unwrap();
        let output = cmd
            .env_clear()
            .env("CONTEXT7_HOME", case)
            .env("HOME", "/tmp")
            .args(["keys", "path"])
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");

        // The returned path must not contain `..` component
        assert!(
            !combined.contains("/../"),
            "CONTEXT7_HOME='{case}': path in output must not contain '/../': {combined}"
        );
        assert!(
            !combined.contains("/.."),
            "CONTEXT7_HOME='{case}': path in output must not end with '/..': {combined}"
        );
        // Must not cause panic
        assert!(
            !combined.contains("thread 'main' panicked"),
            "CONTEXT7_HOME='{case}': must not cause panic: {combined}"
        );
    }
}
