// SPDX-License-Identifier: MIT OR Apache-2.0
// Crate-level lints: documentation hygiene and rustdoc correctness.
#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::private_intra_doc_links)]
//! context7-cli library crate.
//!
//! Exposes the public module hierarchy and the top-level [`run`] entry point
//! used by the binary crate in `src/main.rs`.
//!
//! # Module overview
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`errors`] | Structured error types ([`errors::Context7Error`]) |
//! | [`i18n`] | Bilingual i18n (EN/PT) — [`i18n::Message`] variants and [`i18n::t`] lookup |
//! | [`storage`] | XDG config storage, four-layer key hierarchy, `keys` subcommand operations |
//! | [`api`] | HTTP client, retry-with-rotation, Context7 API calls and response types |
//! | [`output`] | All terminal output — the **only** module allowed to use `println!` |
//! | [`cli`] | Clap structs, subcommand dispatchers |

pub mod api;
pub mod cli;
pub mod errors;
pub mod health;
pub mod i18n;
pub mod output;
pub mod platform;
pub mod storage;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use cli::{Cli, Command};

// ─── LOGGING ─────────────────────────────────────────────────────────────────

/// Wraps the `WorkerGuard` from `tracing-appender`.
///
/// **Must** be kept alive until the end of `main()` to guarantee that the
/// non-blocking log writer flushes its buffer before the process exits.
pub struct LogGuard(#[allow(dead_code)] tracing_appender::non_blocking::WorkerGuard);

impl std::fmt::Debug for LogGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LogGuard").finish()
    }
}

/// Initialises dual logging: terminal (stderr with ANSI) and log file.
///
/// Deletes the previous log file before starting (rotation-by-deletion).
/// Returns [`LogGuard`] — the caller **must** keep it alive until exit.
pub fn init_logging() -> Result<LogGuard> {
    const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

    // Attempt XDG state/log directory; fall back to relative `logs/`
    let logs_dir = storage::discover_xdg_log_paths().unwrap_or_else(|| {
        let compile_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        if compile_root.join("Cargo.toml").exists() {
            compile_root.join("logs")
        } else {
            PathBuf::from("logs")
        }
    });

    let log_path = logs_dir.join(format!("{}.log", BINARY_NAME));

    // Rotation by deletion: remove previous log before initialising
    if log_path.exists() {
        std::fs::remove_file(&log_path)
            .with_context(|| format!("Failed to delete previous log: {}", log_path.display()))?;
    }

    std::fs::create_dir_all(&logs_dir)
        .with_context(|| format!("Failed to create logs directory: {}", logs_dir.display()))?;

    let appender_arquivo =
        tracing_appender::rolling::never(&logs_dir, format!("{}.log", BINARY_NAME));
    let (non_blocking_writer, guard) = tracing_appender::non_blocking(appender_arquivo);

    // Respect RUST_LOG; otherwise default to "context7=info"
    let filtro = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("{}=info", BINARY_NAME)));

    let terminal_layer = tracing_subscriber::fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_writer(std::io::stderr);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_target(true)
        .with_writer(non_blocking_writer);

    tracing_subscriber::registry()
        .with(filtro)
        .with(terminal_layer)
        .with(file_layer)
        .init();

    Ok(LogGuard(guard))
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

/// Main library entry point called from `src/main.rs`.
///
/// Parses CLI arguments, initialises the i18n language setting, then
/// dispatches to the appropriate subcommand handler.
/// Returns `Ok(())` on success or propagates any `anyhow::Error`.
///
/// # Errors
///
/// Returns an [`anyhow::Error`] wrapping any structured
/// [`crate::errors::Context7Error`] or runtime failure. The binary
/// converts the error into a BSD-style exit code via
/// [`crate::errors::Context7Error::exit_code`].
pub async fn run() -> Result<()> {
    let args = Cli::parse();

    // Resolve and lock the UI language as early as possible so every
    // downstream call to `i18n::t()` sees a consistent language.
    let language = i18n::resolve_language(args.lang.as_deref());
    i18n::set_language(language);

    // Respect colour conventions: NO_COLOR (any value) disables
    if std::env::var("NO_COLOR").is_ok() {
        colored::control::set_override(false);
    }
    // CLICOLOR_FORCE=1 forces colours even in pipes
    if std::env::var("CLICOLOR_FORCE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        colored::control::set_override(true);
    }
    // CLI flags have priority over env vars
    if args.no_color || args.json || args.plain {
        colored::control::set_override(false);
    }
    // Suppress stdout when --quiet is passed
    output::set_quiet(args.quiet);

    tokio::select! {
        result = async {
            match args.command {
                Command::Keys { operation } => cli::run_keys(operation, args.json),

                Command::Library { name, query } => cli::run_library(name, query, args.json).await,

                Command::Docs {
                    library_id,
                    query,
                    text,
                } => cli::run_docs(library_id, query, text, args.json).await,

                Command::Completions { shell } => {
                    clap_complete::generate(
                        shell,
                        &mut cli::Cli::command(),
                        "context7",
                        &mut std::io::stdout(),
                    );
                    Ok(())
                }

                Command::Health => {
                    let code = health::run_health(args.json).await?;
                    if code != 0 {
                        std::process::exit(code);
                    }
                    Ok(())
                }
            }
        } => result,
        _ = tokio::signal::ctrl_c() => {
            tracing::warn!("Interrupted by user (Ctrl+C)");
            std::process::exit(130)
        }
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// Smoke test: verify that the Duration import from tokio::time compiles correctly.
    /// This guards against accidental removal of tokio::time re-exports.
    #[test]
    fn test_duration_available() {
        let _ = tokio::time::Duration::from_millis(500);
    }
}
