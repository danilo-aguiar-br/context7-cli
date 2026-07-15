// SPDX-License-Identifier: MIT OR Apache-2.0
//! CLI argument definitions and command dispatchers.
//!
//! Defines [`Cli`], [`Command`], and [`KeysOperation`] via `clap` derives,
//! plus the async dispatcher functions that call into [`crate::api`],
//! [`crate::storage`], and [`crate::output`].
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;

use crate::api::{
    search_library, fetch_documentation, fetch_documentation_text, create_http_client,
    run_with_retry,
};
use crate::errors::Context7Error;
use crate::i18n::{t, Message};
use crate::output::{
    print_libraries_formatted, print_library_not_found_hint,
    print_documentation_formatted, print_json_results, print_plain_text,
};
use crate::storage::{
    load_api_keys, cmd_keys_add, cmd_keys_clear, cmd_keys_export, cmd_keys_import,
    cmd_keys_list, cmd_keys_path, cmd_keys_remove,
};

// ─── CLI STRUCTS ─────────────────────────────────────────────────────────────

/// Top-level CLI entry point parsed by `clap`.
#[derive(Debug, Parser)]
#[command(
    name = "context7",
    version,
    about = "CLI client for the Context7 API (bilingual EN/PT)",
    long_about = None,
)]
pub struct Cli {
    /// UI language: `en` or `pt`. Default: auto-detect from system locale.
    #[arg(long, global = true, env = "CONTEXT7_LANG")]
    pub lang: Option<String>,

    /// Output raw JSON instead of formatted text.
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output (also respected via NO_COLOR env var).
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Plain text output without ANSI formatting (incompatible with --json).
    #[arg(long, global = true, conflicts_with = "json")]
    pub plain: bool,

    /// Increase verbosity (-v info, -vv debug, -vvv trace).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output except errors.
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// Top-level subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Search libraries by name.
    #[command(alias = "lib", alias = "search")]
    Library {
        /// Library name to search for.
        name: String,
        /// Optional context for relevance ranking (e.g. "effect hooks").
        query: Option<String>,
    },

    /// Fetch documentation for a library.
    #[command(alias = "doc", alias = "context")]
    Docs {
        /// Library identifier (e.g. `/rust-lang/rust`).
        library_id: String,

        /// Topic or search query.
        #[arg(short = 'q', long)]
        query: Option<String>,

        /// Output plain text instead of formatted output (incompatible with `--json`).
        #[arg(long, conflicts_with = "json")]
        text: bool,
    },

    /// Manage locally stored API keys.
    #[command(alias = "key")]
    Keys {
        /// Key management operation.
        #[command(subcommand)]
        operation: KeysOperation,
    },

    /// Generate shell completions for bash, zsh, fish, or PowerShell.
    #[command(alias = "completion")]
    Completions {
        /// Shell to generate completions for.
        shell: clap_complete::Shell,
    },

    /// Validate config, keys, and API reachability.
    Health,
}

/// Operations available under the `keys` subcommand.
#[derive(Debug, Subcommand)]
pub enum KeysOperation {
    /// Add an API key to XDG storage.
    Add {
        /// API key to add (e.g. `ctx7sk-abc123…`).
        key: String,
    },
    /// List all stored keys (masked).
    List,
    /// Remove a key by 1-based index (use `keys list` to see indices).
    Remove {
        /// Index of the key to remove (starting at 1).
        index: usize,
    },
    /// Remove all stored keys.
    Clear {
        /// Confirm removal without an interactive prompt.
        #[arg(long)]
        yes: bool,
    },
    /// Print the XDG config file path.
    Path,
    /// Import keys from a `.env` file (reads `CONTEXT7_API=` entries).
    Import {
        /// Path to the `.env` file to import.
        file: std::path::PathBuf,
    },
    /// Export all keys to stdout (one per line, unmasked).
    Export,
}

// ─── INTERNAL HELPERS ────────────────────────────────────────────────────────

/// Shows a hint if the error is `LibraryNotFound`.
fn check_and_show_not_found_hint<T>(result: &anyhow::Result<T>) {
    if let Err(ref e) = result {
        if let Some(Context7Error::LibraryNotFound { .. }) = e.downcast_ref::<Context7Error>()
        {
            print_library_not_found_hint();
        }
    }
}

// ─── DISPATCHERS ─────────────────────────────────────────────────────────────

/// Dispatches `keys` subcommand operations — no HTTP client or API keys needed.
#[must_use]
pub fn run_keys(operation: KeysOperation, json: bool) -> Result<()> {
    match operation {
        KeysOperation::Add { key } => cmd_keys_add(&key),
        KeysOperation::List => cmd_keys_list(json),
        KeysOperation::Remove { index } => cmd_keys_remove(index),
        KeysOperation::Clear { yes } => cmd_keys_clear(yes),
        KeysOperation::Path => cmd_keys_path(),
        KeysOperation::Import { file } => cmd_keys_import(&file),
        KeysOperation::Export => cmd_keys_export(),
    }
}

/// Dispatches the `library` subcommand — searches libraries via the API.
#[must_use]
pub async fn run_library(name: String, query: Option<String>, json: bool) -> Result<()> {
    info!("Searching library: {}", name);

    let keys = load_api_keys()?;
    let client = create_http_client()?;

    info!(
        "Starting context7 with {} API keys available",
        keys.len()
    );

    // API requires the query parameter; fall back to the library name itself
    let context_query = query.as_deref().unwrap_or(&name).to_string();

    let client_arc = std::sync::Arc::new(client);
    // Double-clone needed: outer clone moves ownership to Fn closure,
    // inner clone creates a copy for each retry iteration (closure called N times).
    let name_clone = name.clone();
    let query_clone = context_query.clone();
    let result = run_with_retry(&keys, move |key| {
        let c = std::sync::Arc::clone(&client_arc);
        let n = name_clone.clone();
        let q = query_clone.clone();
        async move { search_library(&c, &key, &n, &q).await }
    })
    .await;

    // Show hint before propagating LibraryNotFound
    check_and_show_not_found_hint(&result);

    let result =
        result.with_context(|| format!("{} '{}'", t(Message::LibrarySearchFailure), name))?;

    if json {
        print_json_results(
            &serde_json::to_string_pretty(&result.results)
                .with_context(|| t(Message::JsonSerialiseFailure))?,
        );
    } else {
        print_libraries_formatted(&result.results);
    }
    Ok(())
}

/// Dispatches the `docs` subcommand — fetches library documentation via the API.
#[must_use]
pub async fn run_docs(
    library_id: String,
    query: Option<String>,
    text: bool,
    json: bool,
) -> Result<()> {
    info!("Fetching documentation for: {}", library_id);

    let keys = load_api_keys()?;
    let client = create_http_client()?;

    info!(
        "Starting context7 with {} API keys available",
        keys.len()
    );

    let client_arc = std::sync::Arc::new(client);
    // Double-clone needed: outer clone moves ownership to Fn closure,
    // inner clone creates a copy for each retry iteration (closure called N times).
    let id_clone = library_id.clone();
    let query_clone = query.clone();

    if text {
        // Plain-text mode: use txt endpoint, print raw markdown
        let text_result = run_with_retry(&keys, move |key| {
            let c = std::sync::Arc::clone(&client_arc);
            let id = id_clone.clone();
            let q = query_clone.clone();
            async move { fetch_documentation_text(&c, &key, &id, q.as_deref()).await }
        })
        .await;

        // Show hint before propagating LibraryNotFound
        check_and_show_not_found_hint(&text_result);

        let text_result = text_result.with_context(|| {
            format!("{} '{}'", t(Message::DocsFetchFailure), library_id)
        })?;

        print_plain_text(&text_result);
        return Ok(());
    }

    // JSON or formatted mode: use json endpoint
    let result = run_with_retry(&keys, move |key| {
        let c = std::sync::Arc::clone(&client_arc);
        let id = id_clone.clone();
        let q = query_clone.clone();
        async move { fetch_documentation(&c, &key, &id, q.as_deref()).await }
    })
    .await;

    // Show hint before propagating LibraryNotFound
    check_and_show_not_found_hint(&result);

    let result = result
        .with_context(|| format!("{} '{}'", t(Message::DocsFetchFailure), library_id))?;

    if json {
        print_json_results(
            &serde_json::to_string_pretty(&result)
                .with_context(|| t(Message::DocsSerialiseFailure))?,
        );
    } else {
        print_documentation_formatted(&result);
    }
    Ok(())
}
