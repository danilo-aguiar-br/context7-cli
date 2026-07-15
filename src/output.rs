// SPDX-License-Identifier: MIT OR Apache-2.0
//! Terminal output formatting.
//!
//! This is the **only** module allowed to call `println!` or `eprintln!`.
/// All coloured formatting via the `colored` crate is centralised here.
/// All user-facing strings are resolved via [`crate::i18n::t`].
use std::io::IsTerminal;
use std::sync::OnceLock;

use anyhow::Context;
use chrono::Utc;
use colored::Colorize;
use serde::Serialize;

use crate::api::{DocumentationSnippet, LibrarySearchResult, DocumentationResponse};
use crate::i18n::{current_language, t, Language, Message};
use crate::storage::StoredKey;

// ─── QUIET MODE ──────────────────────────────────────────────────────────────

static QUIET_MODE: OnceLock<bool> = OnceLock::new();

/// Enables or disables stdout suppression (`--quiet` flag).
///
/// Subsequent calls are silently ignored (OnceLock semantics).
pub fn set_quiet(v: bool) {
    let _ = QUIET_MODE.set(v);
}

fn stdout_allowed() -> bool {
    !QUIET_MODE.get().copied().unwrap_or(false)
}

fn print_line(s: &str) {
    if stdout_allowed() {
        println!("{s}");
    }
}

fn print_blank() {
    if stdout_allowed() {
        println!();
    }
}

/// Returns the Unicode symbol or its ASCII fallback based on terminal capability.
///
/// Uses ASCII when stdout is not an interactive TTY (pipe, redirection),
/// when `NO_COLOR` is set, or when the `TERM` variable is `dumb`.
fn symbol_or_ascii<'a>(unicode: &'a str, ascii: &'a str) -> &'a str {
    static USE_ASCII: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let use_ascii = *USE_ASCII.get_or_init(|| {
        !std::io::stdout().is_terminal()
            || std::env::var("NO_COLOR").is_ok()
            || std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false)
    });
    if use_ascii {
        ascii
    } else {
        unicode
    }
}

// ─── NDJSON ──────────────────────────────────────────────────────────────────

/// NDJSON envelope for structured output consumable by LLMs.
///
/// Each event is emitted as a single JSON line with `type` and `timestamp`.
#[derive(Serialize)]
struct NdjsonEvent<'a, T: Serialize> {
    #[serde(rename = "type")]
    event_type: &'a str,
    timestamp: String,
    #[serde(flatten)]
    data: T,
}

/// Emits an NDJSON event (one JSON line) to stdout.
pub fn emit_ndjson<T: Serialize>(event_type: &str, data: &T) {
    let event = NdjsonEvent {
        event_type,
        timestamp: Utc::now().to_rfc3339(),
        data,
    };
    if let Ok(json) = serde_json::to_string(&event) {
        print_line(&json);
    }
}

// ─── HEALTH ──────────────────────────────────────────────────────────────────

/// Prints a health check status line to stdout, respecting `--quiet`.
pub fn print_health_line(s: &str) {
    print_line(s);
}

/// Returns a colored check/cross symbol for health check results.
///
/// Uses Unicode `✔`/`✘` in TTY mode and ASCII `[OK]`/`[FAIL]` in pipes.
#[must_use]
pub fn health_symbol(ok: bool) -> colored::ColoredString {
    if ok {
        symbol_or_ascii("✔", "[OK]").green()
    } else {
        symbol_or_ascii("✘", "[FAIL]").red()
    }
}

// ─── LIBRARY ─────────────────────────────────────────────────────────────────

/// Prints the list of libraries returned by the search endpoint.
///
/// Displays index, title bold with trust score inline, library ID (dimmed),
/// and optional description (italic).
pub fn print_libraries_formatted(results: &[LibrarySearchResult]) {
    if results.is_empty() {
        print_line(
            &t(Message::NoLibraryFound)
                .yellow()
                .to_string(),
        );
        return;
    }

    print_line(
        &t(Message::LibrariesFound)
            .green()
            .bold()
            .to_string(),
    );
    print_line(&symbol_or_ascii("─", "-").repeat(60).dimmed().to_string());

    for (i, library) in results.iter().enumerate() {
        let number = format!("{}.", i + 1);

        // Title bold with trust score inline
        let title = if let Some(score) = library.trust_score {
            format!(
                "{} {} ({} {:.1}/10)",
                number.cyan(),
                library.title.bold(),
                t(Message::TrustScore),
                score
            )
        } else {
            format!("{} {}", number.cyan(), library.title.bold())
        };
        print_line(&title);

        // ID secondary (dimmed)
        print_line(&format!("   {}", library.id.dimmed()));

        if let Some(description) = &library.description {
            print_line(&format!("   {}", description.italic()));
        }

        print_blank();
    }
}

/// Prints a user-friendly hint when the requested library was not found.
///
/// Called from dispatchers in `cli.rs` before propagating the error,
/// so the user sees the hint on stderr before the error message.
pub fn print_library_not_found_hint() {
    eprintln!("{}", t(Message::LibraryNotFoundApi).yellow());
}

// ─── DOCUMENTATION ───────────────────────────────────────────────────────────

/// Prints structured documentation from the docs endpoint.
///
/// Iterates over `snippets`. Shows a "no documentation found" message if empty.
pub fn print_documentation_formatted(doc: &DocumentationResponse) {
    let snippets = match &doc.snippets {
        Some(s) if !s.is_empty() => s,
        _ => {
            print_line(
                &t(Message::NoDocumentationFound)
                    .yellow()
                    .to_string(),
            );
            return;
        }
    };

    print_line(&t(Message::DocumentationTitle).green().bold().to_string());
    print_line(&symbol_or_ascii("─", "-").repeat(60).dimmed().to_string());

    for snippet in snippets {
        display_snippet(snippet);
    }
}

/// Prints a single documentation snippet with formatted fields.
///
/// Display order: page_title → code_title → code_description → code_list blocks → code_id (source)
fn display_snippet(snippet: &DocumentationSnippet) {
    if let Some(page_title) = &snippet.page_title {
        print_line(&format!("## {}", page_title).green().bold().to_string());
    }

    if let Some(code_title) = &snippet.code_title {
        print_line(
            &format!("{} {}", symbol_or_ascii("▸", ">"), code_title)
                .cyan()
                .to_string(),
        );
    }

    if let Some(description) = &snippet.code_description {
        print_line(&format!("  {}", description.dimmed().italic()));
    }

    if let Some(blocks) = &snippet.code_list {
        for block in blocks {
            print_line(&format!("```{}", block.language));
            print_line(&block.code);
            print_line("```");
        }
    }

    if let Some(source) = &snippet.code_id {
        print_line(&source.blue().bold().dimmed().to_string());
    }

    print_blank();
}

// ─── KEYS ────────────────────────────────────────────────────────────────────

/// Prints all stored keys with 1-based indices and masked values.
pub fn print_masked_keys(keys: &[StoredKey], mask: impl Fn(&str) -> String) {
    print_line(
        &format!("{} {}", keys.len(), t(Message::KeysCount))
            .green()
            .bold()
            .to_string(),
    );
    print_line(&symbol_or_ascii("─", "-").repeat(60).dimmed().to_string());

    let added_label = match current_language() {
        Language::English => "added:",
        Language::Portuguese => "adicionada:",
    };

    for (i, key) in keys.iter().enumerate() {
        print_line(&format!(
            "  {}  {}  {}",
            format!("[{}]", i + 1).cyan(),
            mask(&key.value).bold(),
            format!(
                "({} {})",
                added_label,
                format_added_at_display(&key.added_at)
            )
            .dimmed()
        ));
    }
}

/// Formats an RFC3339 string for compact display: `YYYY-MM-DD HH:MM:SS`.
///
/// Returns the original string if parsing fails (robustness).
pub fn format_added_at_display(iso: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(iso)
        .map_or_else(|_| iso.to_string(), |dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// Prints the "no keys stored" hint message.
pub fn print_no_keys() {
    print_line(&t(Message::NoStoredKey).yellow().to_string());
    print_line(&t(Message::UseKeysAdd).cyan().to_string());
}

/// Prints the "no keys to remove" message.
pub fn print_no_keys_to_remove() {
    print_line(&t(Message::NoKeysToRemove).yellow().to_string());
}

/// Prints an invalid index error.
pub fn print_invalid_index(_index: usize, total: usize) {
    print_line(
        &format!("{} {}.", t(Message::InvalidIndex), total)
            .red()
            .to_string(),
    );
}

/// Prints the success message for `keys add`.
pub fn print_key_added(path: &std::path::Path) {
    print_line(&format!(
        "{} {}",
        t(Message::KeyAdded),
        path.display().to_string().green()
    ));
}

/// Prints the warning message when a key already exists (dedupe).
pub fn print_key_already_existed() {
    print_line(&t(Message::KeyAlreadyExisted).yellow().to_string());
}

/// Displays an error when the user tries to add an empty API key.
pub fn print_invalid_empty_key() {
    eprintln!("{}", t(Message::EmptyOrInvalidKey).red());
}

/// Displays a warning when the key does not match the expected `ctx7sk-` format.
pub fn print_key_format_warning() {
    eprintln!("{}", t(Message::KeyFormatWarning).yellow());
}

/// Prints the success message for `keys remove`.
pub fn print_key_removed(masked_key: &str) {
    print_line(&format!(
        "{} {}",
        masked_key.bold(),
        t(Message::KeyRemovedSuccess)
    ));
}

/// Prints the cancellation message for `keys clear`.
pub fn print_operation_cancelled() {
    print_line(&t(Message::OperationCancelled).yellow().to_string());
}

/// Prints the success message for `keys clear`.
pub fn print_keys_removed() {
    print_line(&t(Message::AllKeysRemoved).green().to_string());
}

/// Prints an "XDG not supported" error for `keys path`.
pub fn print_xdg_unsupported() {
    print_line(&t(Message::XdgSystemNotSupported).red().to_string());
}

/// Prints an empty JSON array `[]` to stdout.
pub fn print_empty_json_array() {
    print_line("[]");
}

/// Prints a raw JSON string to stdout.
pub fn print_raw_json(json: &str) {
    print_line(json);
}

/// Prints a file path to stdout.
pub fn print_config_path(path: &std::path::Path) {
    print_line(&path.display().to_string());
}

/// Prints a key in `CONTEXT7_API=<value>` format to stdout.
pub fn print_exported_key(value: &str) {
    print_line(&format!("CONTEXT7_API={}", value));
}

/// Prints raw JSON results to stdout (used by Library and Docs JSON mode).
pub fn print_json_results(json: &str) {
    print_line(json);
}

/// Prints plain text to stdout (used by Docs text mode).
pub fn print_plain_text(text: &str) {
    print_line(text);
}

/// Asks for interactive confirmation before clearing all keys.
///
/// Returns `true` if the user confirms with `s`/`sim` (Portuguese) or `y`/`yes` (English).
pub fn confirm_clear() -> anyhow::Result<bool> {
    use std::io::Write;
    if stdout_allowed() {
        print!("{}", t(Message::ConfirmRemoveAll));
        std::io::stdout()
            .flush()
            .context("Failed to flush stdout buffer")?;
    }

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .context("Failed to read user confirmation")?;

    Ok(matches!(
        input.trim().to_lowercase().as_str(),
        "s" | "sim" | "y" | "yes"
    ))
}

/// Prints the success message for `keys import`.
pub fn print_import_completed(imported: usize, total: usize) {
    print_line(
        &format!(
            "{}/{} {}",
            imported,
            total,
            t(Message::KeysImportedSuccess)
        )
        .green()
        .to_string(),
    );
}

#[cfg(test)]
mod tests {
    use super::format_added_at_display;

    #[test]
    fn test_format_added_at_rfc3339_with_nanoseconds() {
        let result = format_added_at_display("2026-04-09T13:34:59.060818734+00:00");
        assert_eq!(result, "2026-04-09 13:34:59");
        assert!(
            !result.contains('T'),
            "Result must not contain 'T': {result}"
        );
        assert!(
            !result.contains('.'),
            "Result must not contain nanoseconds: {result}"
        );
        assert!(
            !result.contains("+00:00"),
            "Result must not contain timezone offset: {result}"
        );
    }

    #[test]
    fn test_format_added_at_rfc3339_without_nanoseconds() {
        let result = format_added_at_display("2026-01-01T00:00:00+00:00");
        assert_eq!(result, "2026-01-01 00:00:00");
    }

    #[test]
    fn test_format_added_at_rfc3339_non_utc_offset() {
        // RFC3339 with -03:00 offset (Brazil) — displays local time (no conversion to UTC)
        let result = format_added_at_display("2026-04-09T10:00:00-03:00");
        // The function preserves the local time of the timestamp, does not convert to UTC
        assert_eq!(result, "2026-04-09 10:00:00");
        // Must remove the timezone offset from the display
        assert!(
            !result.contains("-03:00"),
            "Result must not contain timezone offset: {result}"
        );
    }

    #[test]
    fn test_format_added_at_fallback_invalid_string() {
        let result = format_added_at_display("lixo-nao-e-data");
        assert_eq!(
            result, "lixo-nao-e-data",
            "Invalid string must be returned unmodified"
        );
    }

    #[test]
    fn test_format_added_at_empty_string() {
        let result = format_added_at_display("");
        assert_eq!(
            result, "",
            "Empty string must be returned unmodified"
        );
    }

    #[test]
    fn test_format_added_at_output_format_legible() {
        let result = format_added_at_display("2026-04-09T13:34:59.123456789+00:00");
        // Must have exactly the YYYY-MM-DD HH:MM:SS format (19 chars)
        assert_eq!(
            result.len(),
            19,
            "Output format must be 19 characters, got: '{result}'"
        );
        assert!(
            result.contains(' '),
            "Result must contain a space separating date and time: {result}"
        );
    }
}
