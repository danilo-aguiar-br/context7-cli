// SPDX-License-Identifier: MIT OR Apache-2.0
//! Health check subcommand implementation.
//!
//! Validates config, API keys and API reachability.
//!
//! Exit codes follow BSD conventions:
//! - 0  success (all checks passed)
//! - 66 no API keys configured (EX_NOINPUT)
//! - 69 API service unreachable (EX_UNAVAILABLE)
//! - 74 configuration corrupt or unreadable (EX_IOERR)
use std::sync::Arc;

use anyhow::Result;
use colored::Colorize;
use serde::Serialize;
use tokio::time::Duration;
use tracing::info;

use crate::api::{search_library, create_http_client, run_with_retry};
use crate::i18n::{t, Message};
use crate::output::{emit_ndjson, print_health_line, health_symbol};
use crate::storage::load_api_keys;

// ─── TYPES ───────────────────────────────────────────────────────────────────

/// Consolidated result of the health checks.
#[derive(Debug, Serialize)]
pub struct HealthReport {
    /// Whether the local config and HTTP client initialised successfully.
    pub config_ok: bool,
    /// Number of API keys currently configured.
    pub keys_count: usize,
    /// Whether the Context7 API responded during the probe.
    pub api_reachable: bool,
    /// Optional diagnostic detail (e.g. error message, timeout reason).
    pub api_details: Option<String>,
}

// ─── ENTRY POINT ─────────────────────────────────────────────────────────────

/// Runs the health checks and returns the appropriate BSD exit code.
///
/// Sequence: config → keys → API probe (timeout 10s).
#[must_use]
pub async fn run_health(json: bool) -> Result<i32> {
    info!("Running health check");

    if !json {
        print_health_line(&t(Message::HealthRunning).cyan().to_string());
    }

    // ── Check 1: config / HTTP client ──────────────────────────────────────
    let client = match create_http_client() {
        Ok(c) => {
            if !json {
                print_health_line(&format!(
                    "{} {}",
                    health_symbol(true),
                    t(Message::HealthConfigOk).green()
                ));
            }
            c
        }
        Err(err) => {
            let detail = err.to_string();
            if json {
                let report = HealthReport {
                    config_ok: false,
                    keys_count: 0,
                    api_reachable: false,
                    api_details: Some(detail.clone()),
                };
                emit_ndjson("health", &report);
            } else {
                print_health_line(&format!(
                    "{} {} — {}",
                    health_symbol(false),
                    t(Message::HealthConfigFailed).red(),
                    detail
                ));
            }
            return Ok(74);
        }
    };

    // ── Check 2: API keys ───────────────────────────────────────────────
    let keys = match load_api_keys() {
        Ok(c) if !c.is_empty() => {
            if !json {
                print_health_line(&format!(
                    "{} {} {}",
                    health_symbol(true),
                    t(Message::HealthKeysOk).green(),
                    c.len().to_string().bold()
                ));
            }
            c
        }
        _ => {
            if json {
                let report = HealthReport {
                    config_ok: true,
                    keys_count: 0,
                    api_reachable: false,
                    api_details: None,
                };
                emit_ndjson("health", &report);
            } else {
                print_health_line(&format!(
                    "{} {}",
                    health_symbol(false),
                    t(Message::HealthKeysMissing).yellow()
                ));
            }
            return Ok(66);
        }
    };

    // ── Check 3: API probe with 10s timeout ──────────────────────────────────
    let client_arc = Arc::new(client);
    let probe_result = tokio::time::timeout(
        Duration::from_secs(10),
        run_with_retry(&keys, move |key| {
            let c = Arc::clone(&client_arc);
            async move { search_library(&c, &key, "react", "health probe").await }
        }),
    )
    .await;

    let api_reachable = matches!(probe_result, Ok(Ok(_)));
    let api_details = match &probe_result {
        Err(_) => Some("timeout after 10s".to_string()),
        Ok(Err(e)) => Some(e.to_string()),
        Ok(Ok(_)) => None,
    };

    if json {
        let report = HealthReport {
            config_ok: true,
            keys_count: keys.len(),
            api_reachable,
            api_details: api_details.clone(),
        };
        emit_ndjson("health", &report);
    } else if api_reachable {
        print_health_line(&format!(
            "{} {}",
            health_symbol(true),
            t(Message::HealthApiOk).green()
        ));
    } else {
        let detail = api_details.as_deref().unwrap_or("");
        print_health_line(&format!(
            "{} {} — {}",
            health_symbol(false),
            t(Message::HealthApiOffline).red(),
            detail
        ));
    }

    if api_reachable {
        Ok(0)
    } else {
        Ok(69)
    }
}
