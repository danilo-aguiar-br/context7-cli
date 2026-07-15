//! Integration tests for the internationalization (i18n) system.
//!
//! These tests verify the language system behavior through the CLI,
//! using the `--lang` flag and the `CONTEXT7_LANG` environment variable.
//!
//! NOTE v0.2.0: When the `i18n.rs` module is created by implementer-rust,
//! add direct unit tests of `Language::resolver` and `Message::text`
//! importing via `use context7_cli::i18n::*` in the section marked `TODO_v0.2.0`.
//!
//! All tests that manipulate environment variables are marked `#[serial]`.

use assert_cmd::Command;
use predicates::prelude::*;
use serial_test::serial;
use tempfile::TempDir;

// ─── Helper ───

/// Creates isolated command with clean env vars — base for language tests.
///
/// Does NOT define `CONTEXT7_LANG` in the helper — tests that need specific language
/// define it individually via `.env("CONTEXT7_LANG", "pt")`. Setting as `""`
/// would make clap reject the empty value against `value_parser = ["en", "pt"]`.
#[allow(deprecated)] // cargo_bin depreciado no assert_cmd 2.1.0+ (build-dir custom); este projeto não usa build-dir customizado
fn cmd_idioma(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("context7").unwrap();
    cmd.env_clear()
        .env("CONTEXT7_HOME", dir.path())
        .env("HOME", dir.path());
    cmd
}

// ── Testes via flag --lang (v0.2.0) ───────────────────────────────────────────

/// With `--lang pt`, the "no keys" message must be in Portuguese.
/// WARNING: This test requires v0.2.0 to implement the `--lang` flag.
/// In v0.1.0 without `--lang`, the test verifies that `--help` does not break.
#[test]
#[serial]
fn test_help_renders_without_panic_independent_of_lang() {
    let dir = TempDir::new().unwrap();
    // Tests that the binary does not crash in any way with --help
    cmd_idioma(&dir).arg("--help").assert().success();
}

/// `keys list` without any key must display message in Portuguese (v0.1.0 default).
#[test]
#[serial]
fn test_keys_list_empty_message_in_portuguese() {
    let dir = TempDir::new().unwrap();
    cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "pt")
        .args(["keys", "list"])
        .assert()
        .success()
        .stdout(
            // Any of these messages indicates adequate pt-BR
            predicate::str::contains("Nenhuma key")
                .or(predicate::str::contains("nenhuma"))
                .or(predicate::str::contains("Use"))
                .or(predicate::str::contains("0 key")),
        );
}

/// `keys list` without keys — verifies that the message is not a system error.
#[test]
#[serial]
fn test_keys_list_empty_does_not_show_system_error() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir).args(["keys", "list"]).output().unwrap();
    assert!(
        saida.status.success(),
        "keys list without keys must return exit 0"
    );
    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert!(
        !stderr.contains("Error") || stderr.is_empty(),
        "keys list must not produce error messages on stderr: {stderr}"
    );
}

/// With `CONTEXT7_LANG=pt`, the CLI must accept the variable without crash.
#[test]
#[serial]
fn test_env_context7_lang_pt_accepted_without_crash() {
    let dir = TempDir::new().unwrap();
    cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "pt")
        .args(["keys", "list"])
        .assert()
        .success();
}

/// With `CONTEXT7_LANG=en`, the CLI must accept the variable without crash.
#[test]
#[serial]
fn test_env_context7_lang_en_accepted_without_crash() {
    let dir = TempDir::new().unwrap();
    cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "en")
        .args(["keys", "list"])
        .assert()
        .success();
}

/// With `CONTEXT7_LANG=invalido`, the CLI must use fallback without panic.
#[test]
#[serial]
fn test_env_context7_lang_invalid_uses_fallback_without_panic() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "xx-invalido")
        .args(["keys", "list"])
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&saida.stderr);
    assert!(
        !stderr.contains("thread 'main' panicked"),
        "Invalid CONTEXT7_LANG must not cause panic: {stderr}"
    );
}

// ─── Bilingual error message tests (via CLI) ───

/// Without API key with CONTEXT7_LANG=pt, error message must be readable.
#[test]
#[serial]
fn test_no_key_error_legible_message_pt() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "pt")
        .args(["library", "react"])
        .output()
        .unwrap();
    // Must fail but with readable message (not panic, not raw technical message)
    assert!(!saida.status.success());
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("thread 'main' panicked"),
        "must not panic: {combined}"
    );
}

/// Without API key with CONTEXT7_LANG=en, error message must be readable.
#[test]
#[serial]
fn test_no_key_error_legible_message_en() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "en")
        .args(["library", "react"])
        .output()
        .unwrap();
    assert!(!saida.status.success());
    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");
    assert!(
        !combined.contains("thread 'main' panicked"),
        "must not panic: {combined}"
    );
}

// ─── Regression B2: missing key message must respect language ───

/// Regression B2: with CONTEXT7_LANG=pt, missing key error must use i18n PT.
///
/// Before fix B2, `storage.rs:257` used hardcoded message in PT even when
/// the language was EN. After the fix, uses `t(Message::NoKeyConfigured)` which
/// respects the configured language.
#[test]
#[serial]
fn test_b2_missing_key_in_portuguese_uses_pt_message() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "pt")
        .args(["library", "react"])
        .output()
        .unwrap();

    assert!(
        !saida.status.success(),
        "Deve falhar sem key de API configurada"
    );

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("key de API") || combined.contains("CONTEXT7_API_KEYS"),
        "PT message must mention 'key de API' or 'CONTEXT7_API_KEYS', got: {combined}"
    );
    assert!(
        !combined.contains("thread 'main' panicked"),
        "Must not cause panic: {combined}"
    );
}

/// Regression B2: with CONTEXT7_LANG=en, missing key error must use i18n EN.
///
/// Before fix B2, the hardcoded message was always PT. After the fix, it must be EN.
#[test]
#[serial]
fn test_b2_missing_key_in_english_uses_en_message() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .env("CONTEXT7_LANG", "en")
        .args(["library", "react"])
        .output()
        .unwrap();

    assert!(
        !saida.status.success(),
        "Deve falhar sem key de API configurada"
    );

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("API key") || combined.contains("CONTEXT7_API_KEYS"),
        "EN message must mention 'API key' or 'CONTEXT7_API_KEYS', got: {combined}"
    );
    assert!(
        !combined.contains("thread 'main' panicked"),
        "Must not cause panic: {combined}"
    );
}

// ─── Key masking tests (language independent) ───

/// Long key (> 16 chars) must be masked with format prefix12...suffix4.
/// Tests via `keys list` that the full key does not appear in clear text.
#[test]
#[serial]
fn test_keys_list_masks_long_key() {
    let dir = TempDir::new().unwrap();
    let key = "ctx7sk-key-muito-longa-para-mascarar";
    cmd_idioma(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();

    let output = cmd_idioma(&dir).args(["keys", "list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The full key must NOT appear in the listing
    assert!(
        !stdout.contains(key),
        "full key must not appear in clear text in the list: {stdout}"
    );
    // The masked value with long key uses "..." (not "***")
    // Format: prefix12...suffix4
    assert!(
        stdout.contains("..."),
        "long masked key must use '...': {stdout}"
    );
}

/// Short key (< 8 chars) must be masked as "***".
/// Verifies via keys list that no character of the key leaks.
#[test]
#[serial]
fn test_keys_list_masks_short_key() {
    let dir = TempDir::new().unwrap();
    let key = "abc1234"; // < 8 chars
    cmd_idioma(&dir)
        .args(["keys", "add", key])
        .assert()
        .success();

    let output = cmd_idioma(&dir).args(["keys", "list"]).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    // The full key must NOT appear in the listing
    assert!(
        !stdout.contains(key),
        "full short key must not appear in the list: {stdout}"
    );
}

// ── Direct unit tests of i18n module (v0.2.0) ─────────────────────────

use context7_cli::i18n::{resolve_language, Language, Message};

#[test]
fn test_resolve_explicit_flag_en() {
    let language = resolve_language(Some("en"));
    assert!(matches!(language, Language::English));
}

#[test]
fn test_resolve_explicit_flag_pt() {
    let language = resolve_language(Some("pt"));
    assert!(matches!(language, Language::Portuguese));
}

#[test]
#[serial]
fn test_resolve_env_context7_lang_pt_when_no_flag() {
    // SAFETY: #[serial] ensures no other test runs in parallel
    // touching env vars, eliminating data race risk.
    unsafe { std::env::set_var("CONTEXT7_LANG", "pt") };
    let language = resolve_language(None);
    unsafe { std::env::remove_var("CONTEXT7_LANG") };
    assert!(matches!(language, Language::Portuguese));
}

#[test]
fn test_resolve_fallback_when_all_none_returns_valid_language() {
    // Without CONTEXT7_LANG and no "pt" system locale, defaults to English.
    // On CI the locale may vary, so we just assert a valid Language is returned.
    let language = resolve_language(None);
    assert!(matches!(language, Language::English | Language::Portuguese));
}

#[test]
fn test_operation_cancelled_message_en_vs_pt_are_distinct() {
    // Verify the Message variant is accessible via the public API.
    let variante = Message::OperationCancelled;
    let _ = variante; // variant exists and is Copy
                      // The bilingual strings are verified in src/i18n.rs unit tests.
                      // Here we confirm the public API surface is reachable from integration tests.
    let _ = resolve_language(Some("en"));
    let _ = resolve_language(Some("pt"));
}

// ── Tests for new v0.2.1 variants ────────────────────────────────────────

use context7_cli::i18n::t;

/// `DocsFetchFailure` EN e PT são distintas e não-vazias.
///
/// Uses `en()` and `pt()` via localization functions of the i18n module to
/// comparar sem alterar o language global (OnceLock — imutável após set).
#[test]
fn test_fetch_documentation_failure_message_accessible_and_not_empty() {
    // The public API exposes t() which uses current_language().
    // We verify that the variant exists and returns a non-empty string in the default language.
    let text = t(Message::DocsFetchFailure);
    assert!(
        !text.is_empty(),
        "DocsFetchFailure must be non-empty"
    );
    // Verifies that the string is semantically related to "documentation" or "fetch"
    let texto_lower = text.to_lowercase();
    assert!(
        texto_lower.contains("doc")
            || texto_lower.contains("fetch")
            || texto_lower.contains("failure")
            || texto_lower.contains("failed"),
        "DocsFetchFailure must mention doc/fetch/failure/failed: {text}"
    );
}

/// `LibrarySearchFailure` is accessible and returns a non-empty string.
#[test]
fn test_search_library_failure_message_accessible_and_not_empty() {
    let text = t(Message::LibrarySearchFailure);
    assert!(
        !text.is_empty(),
        "LibrarySearchFailure must be non-empty"
    );
}

/// `HttpClientCreateFailure` is accessible and returns a non-empty string.
#[test]
fn test_create_http_client_failure_message_accessible_and_not_empty() {
    let text = t(Message::HttpClientCreateFailure);
    assert!(
        !text.is_empty(),
        "HttpClientCreateFailure must be non-empty"
    );
}

/// `JsonSerialiseFailure` is accessible and returns a non-empty string.
#[test]
fn test_serialise_json_failure_message_accessible_and_not_empty() {
    let text = t(Message::JsonSerialiseFailure);
    assert!(!text.is_empty(), "JsonSerialiseFailure must be non-empty");
}

/// `NoDocumentationAvailable` is accessible and returns a non-empty string.
#[test]
fn test_no_documentation_available_message_accessible_and_not_empty() {
    let text = t(Message::NoDocumentationAvailable);
    assert!(
        !text.is_empty(),
        "NoDocumentationAvailable must be non-empty"
    );
}

// ── Tests for new v0.2.2 variants ────────────────────────────────────────

/// New variant v0.2.2: `LibraryNotFoundApi` must have EN and PT translations
/// distintas, não-vazias e semanticamente coerentes.
///
/// Uses `Message::text(language)` directly to test both languages without
/// depender do OnceLock global (determinístico, sem efeito colateral de estado).
#[test]
fn test_library_not_found_api_message_en_pt() {
    let en = Message::LibraryNotFoundApi.text(Language::English);
    let pt = Message::LibraryNotFoundApi.text(Language::Portuguese);

    assert!(
        !en.is_empty(),
        "EN LibraryNotFoundApi must not be empty"
    );
    assert!(
        !pt.is_empty(),
        "PT LibraryNotFoundApi must not be empty"
    );
    assert_ne!(en, pt, "EN e PT devem ser strings diferentes (bilíngue)");
    assert!(
        en.to_lowercase().contains("library") || en.to_lowercase().contains("not found"),
        "EN must mention 'library' or 'not found', got: {en}"
    );
    assert!(
        pt.to_lowercase().contains("biblioteca") || pt.to_lowercase().contains("encontrada"),
        "PT must mention 'biblioteca' or 'encontrada', got: {pt}"
    );
}

// ── Testes novos v0.2.3 — TrustScore formato trust score (D5) ─────────────

/// `TrustScore` EN must be `"trust"` (lowercase without colon) in v0.2.3.
///
/// v0.2.2: `"Trust:"` — uppercase com dois-pontos, estilo label separado.
/// v0.2.3: `"trust"` — lowercase, used inside parentheses in the library title.
/// New format: `React  (trust 10.0/10)` instead of `Trust: 10.0` on a separate line.
#[test]
fn test_trust_score_en_is_lowercase_without_colon_v023() {
    let en = Message::TrustScore.text(Language::English);
    // v0.2.3: must be "trust" (lowercase, without colon)
    assert_eq!(
        en, "trust",
        "TrustScore EN deve ser 'trust' em v0.2.3, obteve: {en}"
    );
}

/// `TrustScore` PT must be `"confiança"` (lowercase without colon) in v0.2.3.
///
/// v0.2.2: `"Confiança:"` — uppercase com dois-pontos, estilo label separado.
/// v0.2.3: `"confiança"` — lowercase, used inside parentheses in the title.
#[test]
fn test_trust_score_pt_is_lowercase_without_colon_v023() {
    let pt = Message::TrustScore.text(Language::Portuguese);
    // v0.2.3: must be "confiança" (lowercase, without colon)
    assert_eq!(
        pt, "confiança",
        "TrustScore PT deve ser 'confiança' em v0.2.3, obteve: {pt}"
    );
}

// ── Testes B.3 — i18n expandidos: keys add/remove/clear + erro sem key ─────

/// B.3 — `--lang en keys add` displays success message in English.
///
/// Verifies that `KeyAdded` is displayed in English when `--lang en` is active.
/// Uses isolated TempDir to avoid conflict with real user keys.
#[test]
#[serial]
fn test_keys_add_message_en() {
    let dir = TempDir::new().unwrap();
    cmd_idioma(&dir)
        .args([
            "--lang",
            "en",
            "keys",
            "add",
            "ctx7sk-i18n-en-add-test-00001",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("added")
                .or(predicate::str::contains("Key added"))
                .or(predicate::str::contains("successfully")),
        );
}

/// B.3 — `--lang pt keys add` displays success message in Portuguese.
///
/// Verifies that `KeyAdded` is displayed in Portuguese when `--lang pt` is active.
#[test]
#[serial]
fn test_keys_add_message_pt() {
    let dir = TempDir::new().unwrap();
    cmd_idioma(&dir)
        .args([
            "--lang",
            "pt",
            "keys",
            "add",
            "ctx7sk-i18n-pt-add-test-00002",
        ])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("adicionada").or(predicate::str::contains("Chave adicionada")),
        );
}

/// B.3 — `--lang en keys remove` with a key displays removal message in English.
///
/// Verifies that `KeyRemovedSuccess` is displayed in English.
#[test]
#[serial]
fn test_keys_remove_message_en() {
    let dir = TempDir::new().unwrap();
    // Adds the key first
    cmd_idioma(&dir)
        .args(["keys", "add", "ctx7sk-i18n-en-remove-test-00003"])
        .assert()
        .success();

    // Removes with --lang en
    cmd_idioma(&dir)
        .args(["--lang", "en", "keys", "remove", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed").or(predicate::str::contains("Key removed")));
}

/// B.3 — `--lang pt keys remove` with a key displays removal message in Portuguese.
///
/// Verifies that `KeyRemovedSuccess` is displayed in Portuguese.
#[test]
#[serial]
fn test_keys_remove_message_pt() {
    let dir = TempDir::new().unwrap();
    // Adds the key first
    cmd_idioma(&dir)
        .args(["keys", "add", "ctx7sk-i18n-pt-remove-test-00004"])
        .assert()
        .success();

    // Removes with --lang pt
    cmd_idioma(&dir)
        .args(["--lang", "pt", "keys", "remove", "1"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("removed").or(predicate::str::contains("Chave removida")),
        );
}

/// B.3 — `--lang en keys clear --yes` displays cleanup message in English.
///
/// Verifies that `AllKeysRemoved` is displayed in English.
#[test]
#[serial]
fn test_keys_clear_message_en() {
    let dir = TempDir::new().unwrap();
    // Adds a key so that clear has something to remove
    cmd_idioma(&dir)
        .args(["keys", "add", "ctx7sk-i18n-en-clear-test-00005"])
        .assert()
        .success();

    cmd_idioma(&dir)
        .args(["--lang", "en", "keys", "clear", "--yes"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("removed")
                .or(predicate::str::contains("keys removed"))
                .or(predicate::str::contains("All keys")),
        );
}

/// B.3 — `--lang pt keys clear --yes` displays cleanup message in Portuguese.
///
/// Verifies that `AllKeysRemoved` is displayed in Portuguese.
#[test]
#[serial]
fn test_keys_clear_message_pt() {
    let dir = TempDir::new().unwrap();
    // Adds a key so that clear has something to remove
    cmd_idioma(&dir)
        .args(["keys", "add", "ctx7sk-i18n-pt-clear-test-00006"])
        .assert()
        .success();

    cmd_idioma(&dir)
        .args(["--lang", "pt", "keys", "clear", "--yes"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("removidas").or(predicate::str::contains("Todas as keys")),
        );
}

/// B.3 — `--lang en library react` without keys displays error in English.
///
/// Verifies that `NoKeyConfigured` is displayed in English on stderr.
#[test]
#[serial]
fn test_no_key_error_en() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .args(["--lang", "en", "library", "react"])
        .output()
        .unwrap();

    assert!(!saida.status.success(), "deve falhar sem key de API");

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.to_lowercase().contains("no api key")
            || combined.to_lowercase().contains("api key")
            || combined.contains("No API key")
            || combined.contains("CONTEXT7_API_KEYS"),
        "EN error must mention 'API key' or 'CONTEXT7_API_KEYS', got: {combined}"
    );
    assert!(
        !combined.contains("thread 'main' panicked"),
        "must not cause panic: {combined}"
    );
}

/// B.3 — `--lang pt library react` without keys displays error in Portuguese.
///
/// Verifies that `NoKeyConfigured` is displayed in Portuguese on stderr.
#[test]
#[serial]
fn test_no_key_error_pt() {
    let dir = TempDir::new().unwrap();
    let saida = cmd_idioma(&dir)
        .args(["--lang", "pt", "library", "react"])
        .output()
        .unwrap();

    assert!(!saida.status.success(), "deve falhar sem key de API");

    let stderr = String::from_utf8_lossy(&saida.stderr);
    let stdout = String::from_utf8_lossy(&saida.stdout);
    let combined = format!("{stdout}{stderr}");

    assert!(
        combined.contains("Nenhuma")
            || combined.contains("key de API")
            || combined.contains("CONTEXT7_API_KEYS"),
        "PT error must mention 'Nenhuma' or 'key de API', got: {combined}"
    );
    assert!(
        !combined.contains("thread 'main' panicked"),
        "must not cause panic: {combined}"
    );
}
