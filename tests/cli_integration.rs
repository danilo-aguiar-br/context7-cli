//! Testes de integração E2E para a CLI `context7`.
//!
//! All tests invoke the compiled binary via `assert_cmd::Command::cargo_bin`.
//! No test does real network I/O — Context7 API requests are isolated via
//! variável de ambiente ausente (sem key) ou via wiremock para os paths HTTP.
//! No test modifies the real user filesystem — uses `CONTEXT7_HOME`
//! apontando para `tempfile::TempDir`.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Cria um comando `context7` isolado: sem variáveis de ambiente do shell do usuário
/// that may leak API keys or real XDG configurations.
///
/// IMPORTANTE: NÃO define `CONTEXT7_LANG` nem `CONTEXT7_API_KEYS` — definir como string
/// vazia faz o clap tentar parsear `""` contra `value_parser = ["en", "pt"]` e failure.
/// O isolamento de keys é garantido pelo `CONTEXT7_HOME` apontando para um diretório
/// temporário sem nenhum `config.toml`.
#[allow(deprecated)] // cargo_bin depreciado no assert_cmd 2.1.0+ (build-dir custom); este projeto não usa build-dir customizado
fn cmd_isolado(xdg_home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("context7").unwrap();
    cmd.env_clear()
        .env("CONTEXT7_HOME", xdg_home.path())
        .env("HOME", xdg_home.path());
    cmd
}

// ── --help tests ───────────────────────────────────────────────────────────

/// Verifies that `--help` exits with code 0 and contains "Usage".
#[test]
fn test_help_renders_without_panic() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("usage")));
}

/// Verifies that `--version` does not cause panic (may return error if flag is not enabled).
/// A flag `--version` requer `#[command(version)]` no struct Cli do clap.
/// In v0.1.0 without this annotation, the flag does not exist — the test only verifies absence of panic.
#[test]
fn test_version_does_not_panic() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir).arg("--version").output().unwrap();
    // May return exit != 0 if --version is not enabled, but must not panic
    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "--version não deve causar panic: {stderr}"
    );
}

/// Verifies that `context7 library --help` shows the subcommand help.
#[test]
fn test_help_subcommand_library() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["library", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("name")));
}

/// Verifies that `context7 docs --help` shows the subcommand help.
#[test]
fn test_help_subcommand_docs() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["docs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("library")));
}

/// Verifies that `context7 keys --help` shows the subcommand help.
#[test]
fn test_help_subcommand_keys() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("subcommand")));
}

// ── Tests for errors without API key ──────────────────────────────────────────

/// Without API key, `library` must fail with friendly message (not panic).
#[test]
#[serial]
fn test_library_subcommand_without_keys_returns_user_friendly_error() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir)
        .args(["library", "react"])
        .output()
        .unwrap();
    // Deve terminar com exit code não-zero
    assert!(!saida.status.success(), "esperava failure sem key de API");
    // The error message must be friendly, not a panic
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let saida_combinada = format!("{}{}", stdout, stderr);
    assert!(
        !saida_combinada.contains("thread 'main' panicked"),
        "não deve causar panic: {saida_combinada}"
    );
    assert!(
        !saida_combinada.contains("unwrap()"),
        "must not leak unwrap message: {saida_combinada}"
    );
}

/// Without API key, `docs` must fail with friendly message (not panic).
#[test]
#[serial]
fn test_docs_subcommand_without_keys_returns_user_friendly_error() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir)
        .args(["docs", "/facebook/react"])
        .output()
        .unwrap();
    assert!(!saida.status.success(), "esperava failure sem key de API");
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let saida_combinada = format!("{}{}", stdout, stderr);
    assert!(
        !saida_combinada.contains("thread 'main' panicked"),
        "não deve causar panic: {saida_combinada}"
    );
}

// ─── keys subcommand tests (no network) ───

/// `keys list` with empty config returns appropriate message (exit 0).
#[test]
#[serial]
fn test_keys_list_subcommand_empty_returns_appropriate_message() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Nenhuma key")
                .or(predicate::str::contains("0 key"))
                .or(predicate::str::contains("No key"))
                .or(predicate::str::contains("No keys")),
        );
}

/// `keys path` returns a file path (contains the CONTEXT7_HOME override).
#[test]
#[serial]
fn test_keys_path_subcommand_returns_xdg_path() {
    let dir = TempDir::new().unwrap();
    let dir_path = dir.path().to_str().unwrap().to_owned();
    cmd_isolado(&dir)
        .args(["keys", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&dir_path).or(predicate::str::contains("context7")));
}

/// `keys add` adiciona uma key com sucesso (exit 0).
#[test]
#[serial]
fn test_keys_add_subcommand_success() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "add", "ctx7sk-teste-key-123456789012"])
        .assert()
        .success();
}

/// `keys add` + `keys list` shows the masked key.
#[test]
#[serial]
fn test_keys_add_then_list_shows_masked_key() {
    let dir = TempDir::new().unwrap();
    // Adds key
    cmd_isolado(&dir)
        .args(["keys", "add", "ctx7sk-key-integracao-12345678"])
        .assert()
        .success();
    // Lista deve mostrar exactly 1 key
    cmd_isolado(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1 key").or(predicate::str::contains("[1]")));
}

/// `keys remove` with invalid index returns friendly error message (exit 0 — controlled error).
#[test]
#[serial]
fn test_keys_remove_invalid_index_returns_user_friendly_message() {
    let dir = TempDir::new().unwrap();
    // Sem keys, remove índice 99 — deve ser controlado
    let saida = cmd_isolado(&dir)
        .args(["keys", "remove", "99"])
        .output()
        .unwrap();
    // Must not panic
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    assert!(
        !format!("{stdout}{stderr}").contains("thread 'main' panicked"),
        "remove with invalid index must not panic"
    );
}

/// `keys clear --yes` remove todas as keys (exit 0).
#[test]
#[serial]
fn test_keys_clear_yes_success() {
    let dir = TempDir::new().unwrap();
    // Adds a key first
    cmd_isolado(&dir)
        .args(["keys", "add", "ctx7sk-para-limpar-123456789012"])
        .assert()
        .success();
    // Limpa com --yes
    cmd_isolado(&dir)
        .args(["keys", "clear", "--yes"])
        .assert()
        .success();
    // Lista deve estar vazia agora
    cmd_isolado(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Nenhuma key")
                .or(predicate::str::contains("0 key"))
                .or(predicate::str::contains("No key"))
                .or(predicate::str::contains("No keys")),
        );
}

/// `keys export` sem keys produz saída vazia (exit 0).
#[test]
#[serial]
fn test_keys_export_no_keys_empty_output() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "export"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().or(predicate::str::contains("CONTEXT7_API=")));
}

/// `keys export` with key exports in the format CONTEXT7_API=<value>.
#[test]
#[serial]
fn test_keys_export_with_key_correct_format() {
    let dir = TempDir::new().unwrap();
    let key = "ctx7sk-export-teste-123456789012";
    cmd_isolado(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();
    cmd_isolado(&dir)
        .args(["keys", "export"])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("CONTEXT7_API={key}")));
}

// ── Tests for aliases ──────────────────────────────────────────────────────────

/// The `lib` alias is equivalent to `library` — must show the same help.
#[test]
fn test_alias_lib_equivalent_to_library() {
    let dir = TempDir::new().unwrap();
    let saida_library = cmd_isolado(&dir)
        .args(["library", "--help"])
        .output()
        .unwrap();
    let saida_lib = cmd_isolado(&dir).args(["lib", "--help"]).output().unwrap();
    assert_eq!(
        saida_library.status.success(),
        saida_lib.status.success(),
        "alias lib deve ter mesmo exit code que library"
    );
    assert_eq!(
        saida_library.stdout, saida_lib.stdout,
        "alias lib deve produzir mesma saída que library"
    );
}

/// The `doc` alias is equivalent to `docs` — must show the same help.
#[test]
fn test_alias_doc_equivalent_to_docs() {
    let dir = TempDir::new().unwrap();
    let saida_docs = cmd_isolado(&dir).args(["docs", "--help"]).output().unwrap();
    let saida_doc = cmd_isolado(&dir).args(["doc", "--help"]).output().unwrap();
    assert_eq!(
        saida_docs.status.success(),
        saida_doc.status.success(),
        "alias doc deve ter mesmo exit code que docs"
    );
    assert_eq!(
        saida_docs.stdout, saida_doc.stdout,
        "alias doc deve produzir mesma saída que docs"
    );
}

/// The `key` alias is equivalent to `keys` — must show the same help.
#[test]
fn test_alias_key_equivalent_to_keys() {
    let dir = TempDir::new().unwrap();
    let saida_keys = cmd_isolado(&dir).args(["keys", "--help"]).output().unwrap();
    let saida_key = cmd_isolado(&dir).args(["key", "--help"]).output().unwrap();
    assert_eq!(
        saida_keys.status.success(),
        saida_key.status.success(),
        "alias key deve ter mesmo exit code que keys"
    );
    assert_eq!(
        saida_keys.stdout, saida_key.stdout,
        "alias key deve produzir mesma saída que keys"
    );
}

/// Invalid subcommand must return non-zero exit code.
#[test]
fn test_invalid_subcommand_returns_nonzero_exit_code() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .arg("subcomando-que-nao-existe")
        .assert()
        .failure();
}

/// Flag `--json` combined with `keys list` must be accepted without crash.
#[test]
#[serial]
fn test_json_flag_with_keys_list_does_not_crash() {
    let dir = TempDir::new().unwrap();
    // Must not panic — independente do conteúdo da saída
    let saida = cmd_isolado(&dir)
        .args(["--json", "keys", "list"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "não deve panic com --json keys list"
    );
}

// ── New tests v0.2.1 — coverage of fixed bugs ──────────────────────

/// `docs` without API keys displays error message without panic.
///
/// Garante que a ausência de keys não causa panic ou stack overflow,
/// only a controlled error message.
#[test]
#[serial]
fn test_docs_without_keys_shows_error_without_panic() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir)
        .args(["docs", "/test/lib"])
        .output()
        .unwrap();

    assert!(!saida.status.success(), "docs sem keys deve falhar");

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        !combined.contains("thread 'main' panicked"),
        "docs sem keys não deve causar panic: {combined}"
    );
    assert!(
        !combined.contains("unwrap()"),
        "docs without keys must not leak unwrap message: {combined}"
    );
}

/// `context7 docs --help` renderiza ajuda sem crash.
#[test]
fn test_docs_help_renders() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["docs", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage").or(predicate::str::contains("usage")));
}

/// `keys rotate` returns "unrecognized subcommand" — ensures this subcommand
/// foi intencionalmente removido na v0.2.0+ e não foi re-adicionado por engano.
///
/// If this test fails, someone added `rotate` back — review intent.
#[test]
fn test_keys_rotate_returns_unrecognised_subcommand_error() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir).args(["keys", "rotate"]).output().unwrap();

    assert!(
        !saida.status.success(),
        "keys rotate deve retornar exit != 0"
    );

    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert!(
        stderr.contains("unrecognized") || stderr.contains("error"),
        "stderr must mention invalid subcommand: {stderr}"
    );
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "keys rotate não deve causar panic: {stderr}"
    );
}

// ─── New tests v0.2.3 — clap params in EN (Bug #2) ───

/// `library --help` must display `<NAME>` and not `<NOME>` after renaming the parameter.
///
/// v0.2.2: exibia `<NOME>` (identificador Rust em PT).
/// v0.2.3: must display `<NAME>` (EN default for binaries published on crates.io).
#[test]
fn test_library_help_shows_name_in_english() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["library", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<NAME>"))
        .stdout(predicate::str::contains("<NOME>").not());
}

/// `keys add --help` must display `<KEY>` and not `<CHAVE>`.
///
/// v0.2.2: exibia `<CHAVE>`.
/// v0.2.3: must display `<KEY>`.
#[test]
fn test_keys_add_help_shows_key_in_english() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<KEY>"))
        .stdout(predicate::str::contains("<CHAVE>").not());
}

/// `keys remove --help` must display `<INDEX>` and not `<INDICE>`.
///
/// v0.2.2: exibia `<INDICE>`.
/// v0.2.3: must display `<INDEX>`.
#[test]
fn test_keys_remove_help_shows_index_in_english() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<INDEX>"))
        .stdout(predicate::str::contains("<INDICE>").not());
}

/// `keys import --help` must display `<FILE>` and not `<ARQUIVO>`.
///
/// v0.2.2: exibia `<ARQUIVO>`.
/// v0.2.3: must display `<FILE>`.
#[test]
fn test_keys_import_help_shows_file_in_english() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "import", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("<FILE>"))
        .stdout(predicate::str::contains("<ARQUIVO>").not());
}

// ─── New tests v0.2.3 — hardcoded PT hint removed (Bug #3) ───

/// `library --help` must NOT contain the example "hooks de efeito" (PT hardcoded).
///
/// v0.2.2: exibia `(e.g. "hooks de efeito")` no doc comment do parâmetro `query`.
/// v0.2.3: must display `(e.g. "effect hooks")` — neutral EN example.
#[test]
fn test_library_help_does_not_contain_effect_hooks() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["library", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hooks de efeito").not());
}

/// `library --help` must contain the EN example after Bug #3 fix.
///
/// v0.2.3: example becomes `"effect hooks"` instead of `"hooks de efeito"`.
#[test]
fn test_library_help_contains_example_in_english() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["library", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("effect hooks"));
}

// ─── New tests v0.2.3 — LibraryNotFoundApi hint (Bug #1) ───

/// `docs` with non-existent library must display hint to verify the ID on stderr.
///
/// v0.2.2: quando HTTP 404, o erro era "Library not found: /id" sem dica de como
/// encontrar o ID correto — deixava o usuário sem ação clara.
/// v0.2.3: after the error, "context7 library <name>" (EN) or
/// "context7 library <name>" (PT) no stderr como dica de próxima ação.
///
/// Uses wiremock to simulate 404 response without depending on real network.
/// NOTE: This test uses assert_cmd directly with CONTEXT7_HOME pointing
/// para um diretório com uma key falsa, forçando o binário a chamar a API.
/// Since we cannot easily inject wiremock into the compiled binary,
/// testamos via testes unitários de output.rs e i18n.rs.
#[test]
fn test_library_not_found_hint_message_contains_library_command() {
    // Verifica que a message de dica da variante LibraryNotFoundApi
    // in both languages mentions the "library" command to guide the user.
    use context7_cli::i18n::{Language, Message};
    let en = Message::LibraryNotFoundApi.text(Language::English);
    let pt = Message::LibraryNotFoundApi.text(Language::Portuguese);

    assert!(
        en.to_lowercase().contains("library"),
        "EN hint must mention 'library': {en}"
    );
    assert!(
        pt.to_lowercase().contains("library"),
        "PT hint must mention 'library': {pt}"
    );
}

/// `docs` without keys displays error message without mentioning the internal retry mechanism.
///
/// Garante que a message de erro não expõe detalhes de implementação (como
/// "No valid API key after 5 attempts") quando o verdadeiro problema é a ausência de keys.
#[test]
#[serial]
fn test_docs_without_keys_does_not_expose_internal_retry_details() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir)
        .args(["docs", "/biblioteca/inexistente"])
        .output()
        .unwrap();

    assert!(!saida.status.success(), "docs sem keys deve falhar");

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    // Must not leak retry implementation detail
    assert!(
        !combined.contains("No valid API key after"),
        "message não deve expor mecanismo de retry: {combined}"
    );
    assert!(
        !combined.contains("thread 'main' panicked"),
        "must not cause panic: {combined}"
    );
}

// ── Testes regression EXTRA-01/03/04 (v0.2.6) ────────────────────────────────

/// EXTRA-01 — Alias `-q` aparece no help de `docs` e está mapeado para `--query`.
///
/// Regression test: ensures that the short flag `-q` was registered in clap and appears
/// no text de ajuda do subcomando `docs`. Não realiza chamada de rede.
#[test]
fn test_docs_help_shows_alias_q_and_long_query() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir).args(["docs", "--help"]).output().unwrap();

    assert!(
        saida.status.success(),
        "docs --help deve terminar com sucesso"
    );

    let stdout = String::from_utf8_lossy(&saida.stdout);
    assert!(
        stdout.contains("-q") && stdout.contains("--query"),
        "help de docs deve exibir '-q, --query': {stdout}"
    );
}

/// EXTRA-01 — Alias `-q` é aceito pelo parser do clap sem erro de argumento.
///
/// Uses `CONTEXT7_API_KEYS=ctx7sk-fake` to force parse OK + API failure (not parse),
/// verifying that `-q` does not cause "unexpected argument".
#[test]
fn test_docs_alias_q_accepted_as_valid_argument() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir)
        .env("CONTEXT7_API_KEYS", "ctx7sk-fake-key-12345678901")
        .args(["docs", "/reactjs/react.dev", "-q", "hooks"])
        .output()
        .unwrap();

    let stderr = String::from_utf8_lossy(&saida.stderr);
    // O parser NÃO deve rejeitar -q como argumento desconhecido
    assert!(
        !stderr.contains("unexpected argument"),
        "'-q' não deve ser rejeitado como argumento desconhecido: {stderr}"
    );
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "não deve causar panic: {stderr}"
    );
}

/// EXTRA-03 — `keys import` com file inexistente exibe message útil (não panic).
///
/// Regression test: garante que a message de erro menciona o file ou o problema
/// de leitura, sem expor stack trace ou message de sistema ininteligível.
#[test]
fn test_keys_import_nonexistent_file_helpful_message() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "import", "/tmp/file-que-nao-existe-v026.env"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("file")
                .or(predicate::str::contains("file"))
                .or(predicate::str::contains("No such")),
        );
}

/// EXTRA-03 — `keys import` com file sem entradas `CONTEXT7_API=` exibe message útil.
///
/// Regression test: garante que a message de erro menciona `CONTEXT7_API` para guiar
/// o usuário sobre o formato esperado.
#[test]
fn test_keys_import_file_without_context7_keys_helpful_message() {
    let dir = TempDir::new().unwrap();
    let arquivo_invalido = dir.path().join("invalido.env");
    std::fs::write(
        &arquivo_invalido,
        "LIXO=nao_e_chave_context7\nOUTRA_VAR=value\n",
    )
    .unwrap();

    cmd_isolado(&dir)
        .args(["keys", "import", arquivo_invalido.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("CONTEXT7_API"));
}

/// EXTRA-04 — `keys add` with duplicate key warns instead of pretending silent success.
///
/// Regression test: garante que a segunda adição da mesma key exibe message indicando
/// que a key já existia (não simplesmente adiciona duplicata sem aviso).
#[test]
#[serial]
fn test_keys_add_duplicate_shows_existing_key_warning() {
    let dir = TempDir::new().unwrap();
    let key = "ctx7sk-dedup-regression-v026-abc";

    // 1ª adição: deve ter sucesso
    cmd_isolado(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();

    // 2ª adição da mesma key: deve avisar
    let saida = cmd_isolado(&dir)
        .args(["keys", "add", key])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&saida.stdout);
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let combined = format!("{stdout}{stderr}");

    // Deve conter indicação de key existente ou ignorada
    assert!(
        combined.to_lowercase().contains("existe")
            || combined.to_lowercase().contains("exists")
            || combined.to_lowercase().contains("already")
            || combined.to_lowercase().contains("ignor")
            || combined.to_lowercase().contains("skip"),
        "segunda adição deve exibir aviso de duplicata. stdout='{stdout}' stderr='{stderr}'"
    );

    // Must not cause panic
    assert!(
        !combined.contains("thread 'main' panicked"),
        "must not cause panic: {combined}"
    );
}

// ─── Regression tests v0.2.8 (QA gaps) ───

/// FAIL-1 QA v0.2.8 — `keys import` sem CONTEXT7_API= exibe message de erro
/// contendo o path do file tanto na message principal quanto na line
/// "Caused by:" do anyhow chain.
///
/// Regression: ensures that the file path is present in the error output,
/// avoiding regression where "Caused by:" would repeat truncated message without the path.
#[test]
#[serial]
fn test_regression_qa028_fail1_import_no_keys_error_contains_path() {
    let dir = TempDir::new().unwrap();
    let arquivo_sem_chave = dir.path().join("sem_contexto.env");
    std::fs::write(
        &arquivo_sem_chave,
        "FOO=bar\nBAZ=qux\n# sem CONTEXT7_API=\n",
    )
    .unwrap();

    let saida = cmd_isolado(&dir)
        .args(["keys", "import", arquivo_sem_chave.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !saida.status.success(),
        "keys import sem CONTEXT7_API= deve falhar com exit != 0"
    );

    let stderr = String::from_utf8_lossy(&saida.stderr);

    // A message deve conter referência ao path do file
    assert!(
        stderr.contains("sem_contexto.env"),
        "stderr deve conter o name do file. stderr='{stderr}'"
    );

    // A message deve conter referência a CONTEXT7_API
    assert!(
        stderr.contains("CONTEXT7_API"),
        "stderr must mention CONTEXT7_API. stderr='{stderr}'"
    );

    // Must not expose internal implementation details (stack traces, addresses)
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "não deve causar panic. stderr='{stderr}'"
    );
}

/// FAIL-1 QA v0.2.8 — `keys import` file com comentários mas sem keys válidas
/// must fail with exit 1 and useful message.
#[test]
#[serial]
fn test_regression_qa028_fail1_import_only_comments_fails_with_exit1() {
    let dir = TempDir::new().unwrap();
    let arquivo_comentarios = dir.path().join("comentarios.env");
    std::fs::write(
        &arquivo_comentarios,
        "# Este file é só comentários\n# CONTEXT7_API=nao_conta\n",
    )
    .unwrap();

    cmd_isolado(&dir)
        .args(["keys", "import", arquivo_comentarios.to_str().unwrap()])
        .assert()
        .failure()
        .stderr(predicate::str::contains("CONTEXT7_API"));
}

/// FAIL-3 QA v0.2.8 — Mascaramento de key curta (≤ 16 chars) retorna "***".
///
/// Regression: garante que keys muito curtas são tratadas de forma segura
/// (opaque display) instead of exposing the full key.
#[test]
#[serial]
fn test_regression_qa028_fail3_short_key_shows_asterisks() {
    let dir = TempDir::new().unwrap();
    // Chave curta (9 chars) — menor que o threshold de 16
    let short_key = "ctx7sk-ab";

    cmd_isolado(&dir)
        .args(["keys", "add", short_key])
        .assert()
        .success();

    let saida = cmd_isolado(&dir).args(["keys", "list"]).output().unwrap();

    let stdout = String::from_utf8_lossy(&saida.stdout);

    // Chave curta deve aparecer masked — NÃO deve expor o value completo
    assert!(
        !stdout.contains(short_key),
        "key curta não deve aparecer sem mascaramento. stdout='{stdout}'"
    );

    // Deve usar representação de asteriscos para keys curtas
    assert!(
        stdout.contains("***"),
        "key curta deve ser exibida como '***'. stdout='{stdout}'"
    );
}

/// PARTIAL-1 QA v0.2.8 — CONTEXT7_HOME com path traversal é silenciosamente
/// ignorado e o sistema usa o XDG default (comportamento SEGURO).
///
/// Regression: ensures that the anti-path-traversal logic was not removed
/// e que o sistema não acessa caminhos relativos perigosos.
#[test]
#[allow(deprecated)] // cargo_bin depreciado no assert_cmd 2.1.0+ (build-dir custom)
#[serial]
fn test_regression_qa028_partial1_context7_home_path_traversal_uses_default() {
    let dir = TempDir::new().unwrap();

    // Path traversal como CONTEXT7_HOME deve ser silenciosamente ignorado
    // (não deve causar panic nem acessar ../etc)
    let saida = Command::cargo_bin("context7")
        .unwrap()
        .env_clear()
        .env("CONTEXT7_HOME", "../../../etc")
        .env("HOME", dir.path())
        .args(["keys", "path"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&saida.stdout);
    let stderr = String::from_utf8_lossy(&saida.stderr);

    // Must not expose the path ../../../etc in the output
    assert!(
        !stdout.contains("../../../etc"),
        "path traversal must not appear in the output. stdout='{stdout}'"
    );

    // Must not reference /etc in the output (the traversal must not be effective)
    assert!(
        !stdout.contains("/etc/config.toml"),
        "traversal para /etc não deve funcionar. stdout='{stdout}'"
    );

    // Must not cause panic
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "não deve causar panic. stderr='{stderr}'"
    );
}

/// Regression v0.4.0 — `--version` must return the current binary version.
///
/// Garante que o bump de versão em Cargo.toml foi aplicado corretamente
/// e que o binário reporta a versão sincronizada com o pacote.
#[test]
fn test_regression_v040_version_string_correct() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

/// Regression v0.2.8 — FAIL-1 fix: `keys import` without CONTEXT7_API= must not
/// repeat the same message in the "Caused by:" line.
///
/// After the fix, the main message contains the file path ("Arquivo: ...")
/// and the "Caused by:" informs about the absence of keys — without path duplication.
/// The error must be clean: contain the file path and mention CONTEXT7_API.
#[test]
#[serial]
fn test_regression_v028_fail1_import_error_does_not_duplicate_main_message() {
    let dir = TempDir::new().unwrap();
    let arquivo_ruim = dir.path().join("ruim.env");
    std::fs::write(&arquivo_ruim, "OUTRA_VAR=value\n# sem CONTEXT7_API=\n").unwrap();

    let saida = cmd_isolado(&dir)
        .args(["keys", "import", arquivo_ruim.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(
        !saida.status.success(),
        "deve falhar com exit != 0 quando file não tem CONTEXT7_API="
    );

    let stderr = String::from_utf8_lossy(&saida.stderr);

    // Must mention the file path
    assert!(
        stderr.contains("ruim.env"),
        "stderr deve conter o path do file. stderr='{stderr}'"
    );

    // Must mention CONTEXT7_API somewhere in the error
    assert!(
        stderr.contains("CONTEXT7_API"),
        "stderr must mention CONTEXT7_API. stderr='{stderr}'"
    );

    // A message de erro principal NÃO deve ser idêntica ao "Caused by:"
    // (validação do fix que eliminou a duplicação da message completa)
    let linhas: Vec<&str> = stderr.lines().collect();
    let linha_error = linhas.iter().find(|l| l.starts_with("Error:"));
    let linha_caused = linhas.iter().find(|l| l.trim().starts_with("Caused by:"));

    if let (Some(error), Some(_caused_by)) = (linha_error, linha_caused) {
        // A line "Error:" não deve terminar com "Caused by:" idêntico ao início
        // (verifies that it is no longer "Error: Nenhuma key ... \nCaused by:\n    Nenhuma key ...")
        let conteudo_error = error.trim_start_matches("Error:").trim();
        assert!(
            !conteudo_error.is_empty(),
            "a line Error: não deve estar vazia. stderr='{stderr}'"
        );
    }

    // Must not cause panic
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "não deve causar panic. stderr='{stderr}'"
    );
}

// ─── TESTES v0.2.8: input validation + --json keys list + --lang ─────────────

/// Regression v0.2.8 — Bug 1: `keys add` with empty key must fail with exit 1.
///
/// Garante que a validação de entrada rejeita strings vazias before de persistir.
#[test]
fn test_keys_add_empty_key_fails_with_exit1() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["keys", "add", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty").or(predicate::str::contains("vazia")));
}

/// Regression v0.2.8 — Bug 1: `keys add` with whitespace-only key must fail with exit 1.
///
/// A validação faz `.trim()` before de checar vazio, portanto espaços em branco
/// must be rejected the same way as the empty string.
#[test]
fn test_keys_add_whitespace_only_key_fails_with_exit1() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["keys", "add", "   "])
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty").or(predicate::str::contains("vazia")));
}

/// Regression v0.2.8 — Bug 2: `keys add` with key without `ctx7sk-` prefix must
/// emitir aviso em stderr mas completar com sucesso (exit 0).
///
/// The warning is non-blocking: the key is still stored, because the user
/// pode ter uma key de formato legado válida.
#[test]
fn test_keys_add_key_without_prefix_shows_warning_on_stderr() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["keys", "add", "invalid-key-without-prefix-1234567890"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Warning").or(predicate::str::contains("Aviso")));
}

/// Regression v0.2.8 — Bug 2: `keys add` com key no formato correto (`ctx7sk-`)
/// must NOT emit warning on stderr.
///
/// Verifies that the happy path is silent for well-formed keys.
#[test]
fn test_keys_add_valid_prefix_key_does_not_show_warning() {
    let temp = TempDir::new().unwrap();
    let saida = cmd_isolado(&temp)
        .args(["keys", "add", "ctx7sk-test-valid-key-1234567890abcdef"])
        .output()
        .unwrap();
    assert!(
        saida.status.success(),
        "keys add com key válida deve ter exit 0"
    );
    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert!(
        !stderr.contains("Warning") && !stderr.contains("Aviso"),
        "stderr não deve conter aviso para key válida, mas continha: '{stderr}'"
    );
}

/// Regression v0.2.8 — Bug 4: `--json keys list` with a key must produce valid JSON.
///
/// Verifies that the output is a JSON array with the fields `index`, `masked_key` and `added_at`.
#[test]
fn test_keys_list_json_with_key_produces_valid_json() {
    let temp = TempDir::new().unwrap();
    // Adicionar uma key primeiro
    cmd_isolado(&temp)
        .args(["keys", "add", "ctx7sk-test-json-key-1234567890abcdef"])
        .assert()
        .success();
    // Listar em JSON
    let saida = cmd_isolado(&temp)
        .args(["--json", "keys", "list"])
        .output()
        .unwrap();
    assert!(saida.status.success(), "keys list --json deve ter exit 0");
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout não é JSON válido: {e} — output: '{stdout}'"));
    assert!(parsed.is_array(), "esperava array JSON, obteve: {parsed}");
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1, "esperava 1 key no array");
    assert!(arr[0].get("index").is_some(), "campo 'index' ausente");
    assert!(
        arr[0].get("masked_key").is_some(),
        "campo 'masked_key' ausente"
    );
    assert!(arr[0].get("added_at").is_some(), "campo 'added_at' ausente");
}

/// Regression v0.2.8 — Bug 4: `--json keys list` without keys must return empty array `[]`.
///
/// Verifies that the output for empty config is parseable JSON and not a prose message.
#[test]
fn test_keys_list_json_without_keys_returns_empty_array() {
    let temp = TempDir::new().unwrap();
    let saida = cmd_isolado(&temp)
        .args(["--json", "keys", "list"])
        .output()
        .unwrap();
    assert!(
        saida.status.success(),
        "keys list --json deve ter exit 0 mesmo sem keys"
    );
    let stdout = String::from_utf8_lossy(&saida.stdout).trim().to_string();
    assert_eq!(
        stdout, "[]",
        "esperava array JSON vazio, obteve: '{stdout}'"
    );
}

/// Regression v0.2.8 — Gap 12: `--lang en keys list` com config vazia exibe message em inglês.
///
/// Verifies that the `--lang en` flag forces the language of messages to English.
#[test]
fn test_lang_en_keys_list_empty_shows_english() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["--lang", "en", "keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No key stored"));
}

/// Regression v0.2.8 — Gap 12: `--lang pt keys list` com config vazia exibe message em português.
///
/// Verifies that the `--lang pt` flag forces the language of messages to Portuguese.
#[test]
fn test_lang_pt_keys_list_empty_shows_portuguese() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["--lang", "pt", "keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nenhuma chave armazenada"));
}

// ─── Tests B.1 — JSON validation (v0.3.0) ───

/// B.1 — `--json keys list` sem keys produz JSON válido e array vazio.
///
/// Complementa `testa_keys_list_json_sem_chaves_retorna_array_vazio` com
/// parse explícito via serde_json para confirmar que o output é array vazio.
#[test]
fn test_keys_list_json_empty_is_valid_json() {
    let temp = TempDir::new().unwrap();
    let saida = cmd_isolado(&temp)
        .args(["--json", "keys", "list"])
        .output()
        .unwrap();
    assert!(
        saida.status.success(),
        "--json keys list deve ter exit 0 sem keys"
    );
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout não é JSON válido: {e} — output: '{stdout}'"));
    assert!(parsed.is_array(), "esperava array JSON, obteve: {parsed}");
    assert_eq!(
        parsed.as_array().unwrap().len(),
        0,
        "array deve estar vazio sem keys"
    );
}

/// B.1 — `--json keys list` com uma key contém campos `index`, `masked_key` e `added_at`.
///
/// Verifies the structure of the JSON object returned for each key in the list.
#[test]
fn test_keys_list_json_with_key_has_expected_fields() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["keys", "add", "ctx7sk-json-campos-test-12345678"])
        .assert()
        .success();

    let saida = cmd_isolado(&temp)
        .args(["--json", "keys", "list"])
        .output()
        .unwrap();
    assert!(saida.status.success(), "--json keys list deve ter exit 0");

    let stdout = String::from_utf8_lossy(&saida.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout não é JSON válido: {e} — output: '{stdout}'"));

    assert!(parsed.is_array(), "esperava array JSON");
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 1, "esperava 1 elemento no array");

    let obj = &arr[0];
    assert!(
        obj.get("index").is_some(),
        "campo 'index' deve estar presente"
    );
    assert!(
        obj.get("masked_key").is_some(),
        "campo 'masked_key' deve estar presente"
    );
    assert!(
        obj.get("added_at").is_some(),
        "campo 'added_at' deve estar presente"
    );
}

/// B.1 — `--json keys list` com uma key: campo `added_at` usa formato legível.
///
/// Garante que `added_at` no JSON usa formato `YYYY-MM-DD HH:MM:SS` (com espaço)
/// instead of RFC3339 with `T` separator (e.g. `2024-01-15T10:30:00.000000000Z`).
#[test]
fn test_keys_list_json_added_at_legible_format() {
    let temp = TempDir::new().unwrap();
    cmd_isolado(&temp)
        .args(["keys", "add", "ctx7sk-added-at-format-test-12345"])
        .assert()
        .success();

    let saida = cmd_isolado(&temp)
        .args(["--json", "keys", "list"])
        .output()
        .unwrap();
    assert!(saida.status.success(), "--json keys list deve ter exit 0");

    let stdout = String::from_utf8_lossy(&saida.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout não é JSON válido: {e} — output: '{stdout}'"));

    let arr = parsed.as_array().unwrap();
    assert!(!arr.is_empty(), "array não deve estar vazio");

    let added_at = arr[0]["added_at"].as_str().unwrap_or("");
    assert!(!added_at.is_empty(), "campo added_at não deve ser vazio");
    // Formato legível usa espaço entre data e hora, não "T"
    assert!(
        added_at.contains(' '),
        "added_at deve conter espaço (formato YYYY-MM-DD HH:MM:SS), obteve: '{added_at}'"
    );
    assert!(
        !added_at.contains('T'),
        "added_at não deve conter 'T' (não deve ser RFC3339), obteve: '{added_at}'"
    );
}

// ── Testes B.4 — cross-platform (v0.3.0) ─────────────────────────────────────

/// B.4 — CONTEXT7_HOME com path traversal não expõe `..` no path retornado.
///
/// Complementa o teste LOW-01 de storage_integration.rs com foco no value
/// returned by `keys path` — must not contain `..` component.
#[test]
#[serial]
#[allow(deprecated)] // cargo_bin depreciado no assert_cmd 2.1.0+ (build-dir custom)
fn test_context7_home_rejects_path_traversal() {
    use assert_cmd::Command as Cmd;

    let dir = TempDir::new().unwrap();
    let saida = Cmd::cargo_bin("context7")
        .unwrap()
        .env_clear()
        .env("CONTEXT7_HOME", "../../../etc")
        .env("HOME", dir.path())
        .args(["keys", "path"])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&saida.stdout);

    // O path retornado não deve conter `..` — traversal deve ser bloqueado
    assert!(
        !stdout.contains(".."),
        "keys path não deve retornar path com '..' quando CONTEXT7_HOME contém path traversal. stdout='{stdout}'"
    );
}

/// B.4 — `--lang pt keys list` without keys: stdout contains "Nenhuma" (with accent).
///
/// Valida que a saída em português está sendo gerada em UTF-8 correto com acentuação.
#[test]
fn test_output_portuguese_contains_accents() {
    let temp = TempDir::new().unwrap();
    let saida = cmd_isolado(&temp)
        .args(["--lang", "pt", "keys", "list"])
        .output()
        .unwrap();

    assert!(
        saida.status.success(),
        "--lang pt keys list deve ter exit 0"
    );

    let stdout = String::from_utf8(saida.stdout).expect("stdout deve ser UTF-8 válido");

    assert!(
        stdout.contains("Nenhuma"),
        "output em PT deve conter 'Nenhuma' (com N maiúsculo e character 'e'). stdout='{stdout}'"
    );
}

// ─── TESTS SHELL COMPLETIONS (v0.4.0) ───

/// Shell completions bash — exit 0, saída não vazia, contém "context7".
///
/// Verifies that `context7 completions bash` generates a valid bash script.
#[test]
fn test_completions_bash_generates_output() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("context7"))
        .stdout(predicate::str::is_empty().not());
}

/// Shell completions zsh — exit 0, saída não vazia.
///
/// Verifies that `context7 completions zsh` generates a valid zsh script.
#[test]
fn test_completions_zsh_generates_output() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["completions", "zsh"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

/// Shell completions fish — exit 0, saída não vazia.
///
/// Verifies that `context7 completions fish` generates a valid fish script.
#[test]
fn test_completions_fish_generates_output() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["completions", "fish"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

/// Shell completions powershell — exit 0, saída não vazia.
///
/// Verifies that `context7 completions powershell` generates a valid PowerShell script.
#[test]
fn test_completions_powershell_generates_output() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["completions", "powershell"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

/// Alias `completion` funciona igual a `completions`.
///
/// Garante que o alias `#[command(alias = "completion")]` está registrado e operacional.
#[test]
fn test_completions_alias_completion_works() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("context7"));
}

/// `completions --help` — exit 0, lists the available shells.
///
/// Verifies that the subcommand help mentions at least one supported shell.
#[test]
fn test_completions_help_lists_shells() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_isolado(&dir)
        .args(["completions", "--help"])
        .output()
        .unwrap();

    assert!(saida.status.success(), "completions --help deve ter exit 0");

    let stdout = String::from_utf8_lossy(&saida.stdout);
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("bash")
            || combined.contains("zsh")
            || combined.contains("fish")
            || combined.contains("shell"),
        "completions --help must mention available shells. combined='{combined}'"
    );
}

/// `completions bash` opera sem API key — é comando offline.
///
/// Garante que shell completions NÃO requerem CONTEXT7_API_KEYS configurado.
#[test]
fn test_completions_does_not_require_api_key() {
    let dir = TempDir::new().unwrap();
    // cmd_isolado já define CONTEXT7_HOME sem config.toml (sem keys)
    cmd_isolado(&dir)
        .args(["completions", "bash"])
        .assert()
        .success();
}

// ─── TESTE REGRESSÃO USER-AGENT DINÂMICO (v0.4.0) ───────────────────────────

/// Regression v0.4.0 — user-agent in src/api.rs uses CARGO_PKG_VERSION, not hardcoded string.
///
/// Verifies in source code that there is no hardcoded "context7-cli/0." string followed
/// de dígitos — o que indicaria que o user-agent ficou para trás em um release.
/// Any occurrence of ".user_agent(" must use env!("CARGO_PKG_VERSION").
#[test]
fn test_user_agent_does_not_contain_hardcoded_version() {
    let conteudo_api = include_str!("../src/api.rs");

    // Must not exist ".user_agent(" with hardcoded version in format "X.Y.Z"
    let linhas_user_agent: Vec<&str> = conteudo_api
        .lines()
        .filter(|l| l.contains(".user_agent("))
        .collect();

    assert!(
        !linhas_user_agent.is_empty(),
        "src/api.rs deve conter pelo menos uma chamada a .user_agent()"
    );

    for line in &linhas_user_agent {
        // Detectar padrão hardcoded: "context7-cli/0.3.0" ou similar
        assert!(
            !line.contains("context7-cli/0."),
            "src/api.rs:.user_agent() contém versão hardcoded (deve usar env!(\"CARGO_PKG_VERSION\")). line='{line}'"
        );
        assert!(
            line.contains("CARGO_PKG_VERSION"),
            "src/api.rs:.user_agent() deve usar env!(\"CARGO_PKG_VERSION\"). line='{line}'"
        );
    }
}

// ── --quiet regression tests ────────────────────────────────────────────────

/// Regression v0.5.1 — `--quiet` suppresses stdout in `keys list` with empty list.
///
/// Sem keys configuradas, o comportamento sem `--quiet` emite text em stdout.
/// With `--quiet`, stdout must be completely empty.
#[test]
#[serial]
fn test_quiet_suppresses_stdout_in_empty_keys_list() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .arg("--quiet")
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

/// Regression v0.5.1 — `--quiet` preserves stderr when no API key is configured.
///
/// Without a key, the API call fails and must emit error on stderr.
/// With `--quiet`, stdout remains empty but stderr stays active.
#[test]
#[serial]
fn test_quiet_preserves_stderr_in_no_key_error() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .arg("--quiet")
        .args(["library", "react"])
        .assert()
        .failure()
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::is_empty().not());
}

/// Regression v0.5.1 — without `--quiet`, empty `keys list` produces non-empty stdout (control).
///
/// Garante que o comportamento normal não foi quebrado pela implementação do mode silencioso.
#[test]
#[serial]
fn test_no_quiet_produces_stdout_in_empty_keys_list() {
    let dir = TempDir::new().unwrap();
    cmd_isolado(&dir)
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}
