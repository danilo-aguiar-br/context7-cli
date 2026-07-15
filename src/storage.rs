// SPDX-License-Identifier: MIT OR Apache-2.0
//! XDG storage backend for API keys (four-layer hierarchy).
//!
//! Implements the four-layer key loading hierarchy:
//! 1. `CONTEXT7_API_KEYS` runtime environment variable (highest priority)
//! 2. XDG config file `~/.config/context7/config.toml`
//! 3. `.env` file in the current working directory
//! 4. `CONTEXT7_API_KEYS` compile-time environment variable (lowest priority)
use anyhow::{bail, Context, Result};
use chrono::Utc;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use zeroize::{Zeroize, ZeroizeOnDrop};

use unicode_normalization::UnicodeNormalization;

use crate::errors::Context7Error;
use crate::i18n::{t, Message};

// ─── XDG CONFIG STRUCTS ────────────────────────────────────────────

/// Represents a stored API key entry in the XDG configuration file.
///
/// Field names use English (`value`, `added_at`) to mirror the external TOML format.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredKey {
    /// The API key value.
    pub value: String,
    /// RFC 3339 timestamp when the key was added.
    pub added_at: String,
}

/// Represents the structured TOML configuration file.
///
/// Field names use English (`schema_version`, `keys`) to mirror the external TOML format.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FileConfig {
    /// Configuration schema version (currently 1).
    pub schema_version: u32,
    /// List of stored API keys.
    #[serde(default)]
    pub keys: Vec<StoredKey>,
}

// ─── SECURE NEWTYPE FOR API KEYS ─────────────────────────────────────────

/// Secure wrapper for API keys with automatic memory cleanup.
///
/// Implements `Zeroize` and `ZeroizeOnDrop` to ensure keys are
/// removed from memory when leaving scope. `Debug` and `Display` show
/// only the masked version of the key to prevent leakage in logs.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ApiKey(String);

impl ApiKey {
    /// Creates a new instance from a key string.
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Returns a reference to the inner key value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

impl PartialEq<&str> for ApiKey {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ApiKey({})", mask_key(self.value()))
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", mask_key(self.value()))
    }
}

// ─── FILE PERMISSIONS FUNCTIONS ───────────────────────────────────────

/// Sets 600 permissions (owner read/write only) on Unix systems.
///
/// On non-Unix systems this is a no-op. Centralises the chmod 600 used by
/// [`write_xdg_config`] and [`write_file_config`].
#[must_use]
pub fn apply_600_permissions(path: &std::path::Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)
            .with_context(|| format!("Failed to read metadata of: {}", path.display()))?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to set permissions on: {}", path.display()))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

// ─── XDG PATH DISCOVERY FUNCTIONS ──────────────────────────────────

/// Resolves a base path override from the `CONTEXT7_HOME` environment variable.
///
/// Returns `Some(PathBuf)` if `CONTEXT7_HOME` is set, non-empty, and contains no
/// path-traversal components (`..`). Returns `None` otherwise, allowing callers
/// to fall back to XDG/ProjectDirs defaults.
fn resolve_home_override() -> Option<PathBuf> {
    let home = std::env::var("CONTEXT7_HOME").ok()?;
    if home.is_empty() {
        return None;
    }
    let base = PathBuf::from(&home);
    // Reject path traversal to prevent escaping the configuration directory
    if base
        .components()
        .any(|c| c == std::path::Component::ParentDir)
    {
        tracing::warn!(
            "CONTEXT7_HOME='{}' rejected (path traversal) — using XDG default",
            home
        );
        return None;
    }
    // Reject Windows reserved names in any path component
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT0", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8",
        "LPT9",
    ];
    for component in base.components() {
        if let std::path::Component::Normal(name) = component {
            let name_upper = name.to_string_lossy().to_uppercase();
            // Check pure name and name with extension (e.g. NUL.txt)
            let base_name = name_upper.split('.').next().unwrap_or("");
            if reserved_names.contains(&base_name) {
                tracing::warn!(
                    "CONTEXT7_HOME='{}' rejected (Windows reserved name '{}') — using XDG default",
                    home, base_name
                );
                return None;
            }
        }
    }
    Some(base)
}

/// Discovers the XDG configuration path for the `config.toml` file.
///
/// Checks `CONTEXT7_HOME` first: if set and valid, returns
/// `{CONTEXT7_HOME}/context7/config.toml`. Falls back to
/// `ProjectDirs::from("", "", "context7")` → `config_dir()`.
/// Returns `None` if neither source provides a path.
#[must_use]
pub fn discover_config_path() -> Option<PathBuf> {
    let path = if let Some(base) = resolve_home_override() {
        base.join("context7").join("config.toml")
    } else {
        ProjectDirs::from("", "", "context7")?
            .config_dir()
            .join("config.toml")
    };
    // Normalise to NFC — macOS HFS+ uses NFD by default
    let path_str = path.to_string_lossy().nfc().collect::<String>();
    Some(PathBuf::from(path_str))
}

/// Discovers the XDG path for storing log files.
///
/// Checks `CONTEXT7_HOME` first: if set and valid, returns
/// `{CONTEXT7_HOME}/context7/logs`. Falls back to `state_dir()` on Linux
/// (XDG_STATE_HOME) with fallback to `data_local_dir()`.
/// Returns `None` if neither source provides a path.
#[must_use]
pub fn discover_xdg_log_paths() -> Option<PathBuf> {
    let path = if let Some(base) = resolve_home_override() {
        base.join("context7").join("logs")
    } else {
        let dirs = ProjectDirs::from("", "", "context7")?;
        // state_dir() is only available on Linux/XDG; cross-platform fallback
        #[cfg(target_os = "linux")]
        {
            dirs.state_dir()
                .unwrap_or_else(|| dirs.data_local_dir())
                .to_path_buf()
        }
        #[cfg(not(target_os = "linux"))]
        {
            dirs.data_local_dir().to_path_buf()
        }
    };
    // Normalise to NFC — macOS HFS+ uses NFD by default
    let path_str = path.to_string_lossy().nfc().collect::<String>();
    Some(PathBuf::from(path_str))
}

// ─── KEY LOADING FUNCTIONS (HIERARCHY) ─────────────────────────

/// Layer 1: reads keys from the `CONTEXT7_API_KEYS` runtime environment variable.
///
/// Accepts multiple comma-separated keys:
/// `CONTEXT7_API_KEYS=ctx7sk-a,ctx7sk-b,ctx7sk-c`
/// Whitespace around each key is trimmed automatically.
/// Returns `None` if the variable is not set or is empty.
#[must_use]
pub fn read_env_var_key() -> Option<Vec<String>> {
    std::env::var("CONTEXT7_API_KEYS")
        .ok()
        .map(|value| {
            let estimate = value.matches(',').count() + 1;
            let mut keys = Vec::with_capacity(estimate);
            for s in value.split(',') {
                let trimmed = s.trim().to_string();
                if !trimmed.is_empty() {
                    keys.push(trimmed);
                }
            }
            keys
        })
        .filter(|v| !v.is_empty())
}

/// Layer 2: reads keys from the XDG configuration file (`config.toml`).
///
/// Returns `None` if the file does not exist or the XDG path is unavailable.
/// Returns `Err` if the file exists but contains invalid TOML.
#[must_use]
pub fn read_xdg_config() -> Result<Option<Vec<String>>> {
    let path = match discover_config_path() {
        Some(p) => p,
        None => return Ok(None),
    };

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read XDG configuration at: {}", path.display()))?;

    let config: FileConfig = toml::from_str(&content)
        .with_context(|| format!("Invalid TOML at: {}", path.display()))?;

    let keys: Vec<String> = config
        .keys
        .into_iter()
        .map(|c| c.value)
        .filter(|v| !v.is_empty())
        .collect();

    if keys.is_empty() {
        Ok(None)
    } else {
        Ok(Some(keys))
    }
}

/// Layer 3: reads keys from the `.env` file in the current working directory.
///
/// Re-uses [`extract_env_keys`] which is pure and testable.
/// Returns `None` if the `.env` file does not exist or has no valid keys.
#[must_use]
pub fn read_env_cwd() -> Option<Vec<String>> {
    let path = std::env::current_dir().ok().map(|d| d.join(".env"))?;

    if !path.exists() {
        return None;
    }

    std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| extract_env_keys(&content).ok())
}

/// Layer 4: reads keys embedded at compile time via `option_env!("CONTEXT7_API_KEYS")`.
///
/// Allows embedding keys in the binary at build time:
/// `CONTEXT7_API_KEYS=ctx7sk-a cargo build --release`
///
/// **Security warning**: compile-time keys are visible to anyone who inspects
/// the binary (e.g. `strings context7 | grep ctx7sk-`). Use only in controlled
/// pipelines where access to the binary artefact is restricted.
///
/// Returns `None` if the variable was not defined at compile time.
#[must_use]
pub fn read_compile_time_env() -> Option<Vec<String>> {
    option_env!("CONTEXT7_API_KEYS").map(|value| {
        let estimate = value.matches(',').count() + 1;
        let mut keys = Vec::with_capacity(estimate);
        for s in value.split(',') {
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                keys.push(trimmed);
            }
        }
        keys
    })
}

/// Loads API keys using the four-layer precedence hierarchy:
///
/// 1. `CONTEXT7_API_KEYS` runtime env var (highest priority)
/// 2. XDG config `~/.config/context7/config.toml`
/// 3. `.env` file in the current working directory
/// 4. `CONTEXT7_API_KEYS` compile-time env var (lowest priority)
///
/// Returns an error only if NO layer provides valid keys.
#[must_use]
pub fn load_api_keys() -> Result<Vec<ApiKey>> {
    use tracing::{info, warn};

    // Layer 1: runtime env var
    if let Some(keys) = read_env_var_key() {
        info!("Keys loaded from CONTEXT7_API_KEYS environment variable");
        return Ok(keys.into_iter().map(ApiKey::new).collect());
    }

    // Layer 2: XDG config
    match read_xdg_config() {
        Ok(Some(keys)) => {
            info!("Keys loaded from XDG configuration");
            return Ok(keys.into_iter().map(ApiKey::new).collect());
        }
        Ok(None) => {}
        Err(e) => {
            warn!("Failed to read XDG configuration (continuing): {}", e);
        }
    }

    // Layer 3: .env in CWD
    if let Some(keys) = read_env_cwd() {
        info!(
            "Starting context7 with {} API keys available",
            keys.len()
        );
        return Ok(keys.into_iter().map(ApiKey::new).collect());
    }

    // Layer 4: compile-time
    if let Some(keys) = read_compile_time_env() {
        info!("Keys loaded from compile-time CONTEXT7_API_KEYS");
        return Ok(keys.into_iter().map(ApiKey::new).collect());
    }

    bail!(t(Message::NoKeyConfigured))
}

// ─── CONFIG WRITE FUNCTIONS ───────────────────────────────────────────

/// Writes (or updates) the XDG configuration file with the provided key.
///
/// Creates parent directories if necessary.
/// On Unix systems, sets 600 permissions via [`apply_600_permissions`].
#[must_use]
pub fn write_xdg_config(new_key: &str) -> Result<PathBuf> {
    let path = discover_config_path()
        .context("System does not support XDG directories — cannot save configuration")?;

    // Create parent directories if they do not exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Read existing config or create a new one
    let mut config = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read existing config: {}", path.display()))?;
        toml::from_str::<FileConfig>(&content)
            .with_context(|| format!("Invalid TOML at: {}", path.display()))?
    } else {
        FileConfig {
            schema_version: 1,
            keys: Vec::new(),
        }
    };

    // Add new key if it does not already exist
    let already_exists = config.keys.iter().any(|c| c.value == new_key);
    if !already_exists {
        config.keys.push(StoredKey {
            value: new_key.to_string(),
            added_at: Utc::now().to_rfc3339(),
        });
    }

    // Serialise and write
    let toml_str =
        toml::to_string_pretty(&config).context("Failed to serialise configuration to TOML")?;
    std::fs::write(&path, &toml_str)
        .with_context(|| format!("Failed to write config at: {}", path.display()))?;

    apply_600_permissions(&path)?;

    Ok(path)
}

/// Reads the XDG configuration file and returns the full [`FileConfig`].
///
/// Used by operations that need the complete structure (list, remove, export).
/// Returns `Ok(None)` if the file does not exist or the XDG path is unavailable.
#[must_use]
pub fn read_xdg_config_raw() -> Result<Option<FileConfig>> {
    let path = match discover_config_path() {
        Some(p) => p,
        None => return Ok(None),
    };

    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read XDG configuration at: {}", path.display()))?;

    let config: FileConfig = toml::from_str(&content)
        .with_context(|| format!("Invalid TOML at: {}", path.display()))?;

    Ok(Some(config))
}

/// Writes a complete [`FileConfig`] to the XDG configuration file.
///
/// Creates parent directories if necessary.
/// On Unix systems, sets 600 permissions.
#[must_use]
pub fn write_file_config(config: &FileConfig) -> Result<PathBuf> {
    let path = discover_config_path()
        .context("System does not support XDG directories — cannot save configuration")?;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    let toml_str =
        toml::to_string_pretty(config).context("Failed to serialise configuration to TOML")?;
    std::fs::write(&path, &toml_str)
        .with_context(|| format!("Failed to write config at: {}", path.display()))?;

    apply_600_permissions(&path)?;

    Ok(path)
}

// ─── AUXILIARY FUNCTIONS ─────────────────────────────────────────────────────

/// Masks an API key showing only the first 12 and last 4 characters.
///
/// Example: `ctx7sk-abc123...xyz9`
///
/// If the key is too short (≤ 16 Unicode characters), returns `***` for protection.
/// Uses `chars()` for UTF-8 safety — avoids panics from byte-indexing multibyte characters.
#[must_use]
pub fn mask_key(key: &str) -> String {
    let n_chars = key.chars().count();
    let prefix_len = 12;
    let suffix_len = 4;
    if n_chars <= prefix_len + suffix_len {
        return "***".to_string();
    }
    let prefix: String = key.chars().take(prefix_len).collect();
    let suffix: String = key
        .chars()
        .rev()
        .take(suffix_len)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{}...{}", prefix, suffix)
}

/// Extracts `CONTEXT7_API=` keys from `.env` file content in memory.
///
/// Ignores comments (lines starting with `#`) and blank lines.
/// Removes surrounding double and single quotes from values.
/// Pure function — accepts `&str`, no I/O, facilitates unit testing.
#[must_use]
pub fn extract_env_keys(content: &str) -> Result<Vec<String>> {
    let keys: Vec<String> = content
        .lines()
        .filter_map(|line| {
            // Remove inline comments (everything after #)
            let line_no_comment = line.split('#').next().unwrap_or("").trim();
            line_no_comment
                .strip_prefix("CONTEXT7_API=")
                .map(|value| {
                    // Remove single or double quotes around the value
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string()
                })
                .filter(|v| !v.is_empty())
        })
        .collect();

    if keys.is_empty() {
        bail!(t(Message::NoContext7KeyInFile));
    }

    Ok(keys)
}

// ─── KEYS SUBCOMMAND OPERATIONS ───────────────────────────────────────────

/// Adds a new key to the XDG storage.
///
/// If the key already exists, prints a warning and returns without modifying the config.
/// Re-uses [`write_xdg_config`] which implements deduplication and chmod 600.
#[must_use]
pub fn cmd_keys_add(key: &str) -> Result<()> {
    let trimmed_key = key.trim();
    if trimmed_key.is_empty() {
        crate::output::print_invalid_empty_key();
        bail!(Context7Error::KeysOperationFailed);
    }
    if !trimmed_key.starts_with("ctx7sk-") || trimmed_key.len() < 16 {
        crate::output::print_key_format_warning();
    }
    // Check for duplicates before writing — to display a clear warning to the user
    if let Some(config) = read_xdg_config_raw()? {
        if config.keys.iter().any(|c| c.value == trimmed_key) {
            crate::output::print_key_already_existed();
            return Ok(());
        }
    }
    let path = write_xdg_config(trimmed_key)?;
    crate::output::print_key_added(&path);
    Ok(())
}

/// Lists all stored keys with their 1-based indices and masked values.
///
/// When `json` is true, outputs a JSON array with `index`, `masked_key`, and `added_at` fields.
#[must_use]
pub fn cmd_keys_list(json: bool) -> Result<()> {
    match read_xdg_config_raw()? {
        None => {
            if json {
                crate::output::print_empty_json_array();
            } else {
                crate::output::print_no_keys();
            }
        }
        Some(config) if config.keys.is_empty() => {
            if json {
                crate::output::print_empty_json_array();
            } else {
                crate::output::print_no_keys();
            }
        }
        Some(config) => {
            if json {
                let mut masked: Vec<serde_json::Value> = Vec::with_capacity(config.keys.len());
                masked.extend(config.keys.iter().enumerate().map(|(i, k)| {
                    serde_json::json!({
                        "index": i + 1,
                        "masked_key": mask_key(&k.value),
                        "added_at": crate::output::format_added_at_display(&k.added_at)
                    })
                }));
                crate::output::print_raw_json(
                    &serde_json::to_string_pretty(&masked).with_context(|| {
                        crate::i18n::t(crate::i18n::Message::JsonSerialiseFailure)
                    })?,
                );
            } else {
                crate::output::print_masked_keys(&config.keys, mask_key);
            }
        }
    }
    Ok(())
}

/// Removes a key by its 1-based index.
#[must_use]
pub fn cmd_keys_remove(index: usize) -> Result<()> {
    let mut config = match read_xdg_config_raw()? {
        None => {
            crate::output::print_no_keys_to_remove();
            bail!(Context7Error::KeysOperationFailed);
        }
        Some(c) if c.keys.is_empty() => {
            crate::output::print_no_keys_to_remove();
            bail!(Context7Error::KeysOperationFailed);
        }
        Some(c) => c,
    };

    if index == 0 || index > config.keys.len() {
        crate::output::print_invalid_index(index, config.keys.len());
        bail!(Context7Error::KeysOperationFailed);
    }

    let removed = config.keys.remove(index - 1);
    write_file_config(&config)?;
    crate::output::print_key_removed(&mask_key(&removed.value));
    Ok(())
}

/// Removes all stored keys. Asks for confirmation unless `--yes` is passed.
#[must_use]
pub fn cmd_keys_clear(yes: bool) -> Result<()> {
    if !yes && !crate::output::confirm_clear()? {
        crate::output::print_operation_cancelled();
        return Ok(());
    }

    let config = FileConfig {
        schema_version: 1,
        keys: Vec::new(),
    };
    write_file_config(&config)?;
    crate::output::print_keys_removed();
    Ok(())
}

/// Displays the path of the XDG configuration file.
#[must_use]
#[allow(clippy::unnecessary_wraps)]
pub fn cmd_keys_path() -> Result<()> {
    match discover_config_path() {
        Some(path) => crate::output::print_config_path(&path),
        None => crate::output::print_xdg_unsupported(),
    }
    Ok(())
}

/// Imports keys from a `.env` file, reading `CONTEXT7_API=` entries.
///
/// Re-uses [`extract_env_keys`] and [`write_xdg_config`] for each key.
#[must_use]
pub fn cmd_keys_import(file: &std::path::Path) -> Result<()> {
    let content = std::fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let keys =
        extract_env_keys(&content).with_context(|| format!("File: {}", file.display()))?;

    let total = keys.len();
    let mut imported = 0usize;

    for key in &keys {
        write_xdg_config(key)?;
        imported += 1;
    }

    crate::output::print_import_completed(imported, total);
    Ok(())
}

/// Exports all keys to stdout in `CONTEXT7_API=<value>` format, one per line.
///
/// Compatible with `.env` files — useful for scripts and pipes.
#[must_use]
pub fn cmd_keys_export() -> Result<()> {
    match read_xdg_config_raw()? {
        None => {}
        Some(config) if config.keys.is_empty() => {}
        Some(config) => {
            for key in &config.keys {
                crate::output::print_exported_key(&key.value);
            }
        }
    }
    Ok(())
}

// ─── TESTS ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Test helper function ──────────────────────────────────────────────

    /// Reads the content of a TOML file from the path and returns `FileConfig`.
    fn read_toml_config_from_path(path: &std::path::Path) -> Result<FileConfig> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("Invalid TOML at: {}", path.display()))
    }

    // ── Parsing do .env ───────────────────────────────────────────────────────

    #[test]
    fn test_env_parsing_with_multiple_equal_keys() {
        let mut content = String::new();
        for i in 0..17 {
            content.push_str(&format!("CONTEXT7_API=ctx7sk-key-{:02}\n", i));
        }
        let keys = extract_env_keys(&content).expect("Must extract 17 keys without error");
        assert_eq!(keys.len(), 17, "Must return exactly 17 keys");
        for (i, key) in keys.iter().enumerate() {
            assert_eq!(
                key,
                &format!("ctx7sk-key-{:02}", i),
                "Chave {} deve ter o value correto",
                i
            );
        }
    }

    #[test]
    fn test_env_parsing_ignores_comments_and_blank_lines() {
        let content = "# Este é um comentário\n\
                        CONTEXT7_API=ctx7sk-key-valida-01\n\
                        \n\
                        # Outro comentário\n\
                        CONTEXT7_API=ctx7sk-key-valida-02\n\
                        \n";
        let keys = extract_env_keys(content).expect("Must extract keys without error");
        assert_eq!(keys.len(), 2, "Must ignore comments and blank lines");
        assert_eq!(keys[0], "ctx7sk-key-valida-01");
        assert_eq!(keys[1], "ctx7sk-key-valida-02");
    }

    #[test]
    fn test_env_parsing_removes_double_quotes() {
        let content = "CONTEXT7_API=\"ctx7sk-abc-com-aspas\"\n";
        let keys = extract_env_keys(content).expect("Must extract key without error");
        assert_eq!(keys.len(), 1);
        assert_eq!(
            keys[0], "ctx7sk-abc-com-aspas",
            "Must remove double quotes"
        );
    }

    #[test]
    fn test_env_parsing_removes_single_quotes() {
        let content = "CONTEXT7_API='ctx7sk-abc-aspas-simples'\n";
        let keys = extract_env_keys(content).expect("Must extract key without error");
        assert_eq!(keys.len(), 1);
        assert_eq!(
            keys[0], "ctx7sk-abc-aspas-simples",
            "Must remove single quotes"
        );
    }

    #[test]
    fn test_env_parsing_error_when_no_keys() {
        let content = "# Apenas comentários\n\
                        OUTRA_VAR=value\n\
                        \n";
        let result = extract_env_keys(content);
        assert!(
            result.is_err(),
            "Must return Err when there are no CONTEXT7_API keys"
        );
        let mensagem_erro = result.unwrap_err().to_string();
        assert!(
            mensagem_erro.contains("key")
                || mensagem_erro.contains("CONTEXT7_API")
                || mensagem_erro.contains("key")
                || mensagem_erro.contains("API"),
            "Message de erro deve mencionar CONTEXT7_API, key, key ou API, obteve: {}",
            mensagem_erro
        );
    }

    #[test]
    fn test_env_parsing_ignores_empty_keys() {
        let content = "CONTEXT7_API=\n\
                        CONTEXT7_API=ctx7sk-valida\n";
        let keys = extract_env_keys(content).expect("Must extract key without error");
        assert_eq!(
            keys.len(),
            1,
            "Must ignore CONTEXT7_API entries without value"
        );
        assert_eq!(keys[0], "ctx7sk-valida");
    }

    #[test]
    fn test_env_parsing_ignores_inline_comment() {
        let content = "CONTEXT7_API=ctx7sk-valida # comentário aqui\n";
        let keys = extract_env_keys(content).expect("Must extract key without error");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "ctx7sk-valida");
    }

    // ── B.2: CRLF line endings ────────────────────────────────────────────────

    #[test]
    fn test_env_parsing_with_crlf_line_endings() {
        // Arquivo .env gerado no Windows usa \r\n
        let content = "CONTEXT7_API=ctx7sk-crlf-key-a\r\nCONTEXT7_API=ctx7sk-crlf-key-b\r\n";
        let keys =
            extract_env_keys(content).expect("Must extract 2 keys from CRLF content without error");
        assert_eq!(
            keys.len(),
            2,
            "Must return exactly 2 keys with CRLF"
        );
        assert_eq!(
            keys[0], "ctx7sk-crlf-key-a",
            "Primeira key não deve conter \\r residual"
        );
        assert_eq!(
            keys[1], "ctx7sk-crlf-key-b",
            "Segunda key não deve conter \\r residual"
        );
    }

    #[test]
    fn test_env_parsing_with_mixed_line_endings() {
        // Mix de LF (\n) e CRLF (\r\n) no mesmo file
        let content = "CONTEXT7_API=ctx7sk-mixed-key-a\nCONTEXT7_API=ctx7sk-mixed-key-b\r\n";
        let keys = extract_env_keys(content)
            .expect("Must extract 2 keys from mixed LF/CRLF content without error");
        assert_eq!(
            keys.len(),
            2,
            "Must return exactly 2 keys with mixed line endings"
        );
        assert_eq!(
            keys[0], "ctx7sk-mixed-key-a",
            "Chave com LF não deve ter \\r residual"
        );
        assert_eq!(
            keys[1], "ctx7sk-mixed-key-b",
            "Chave com CRLF não deve ter \\r residual"
        );
    }

    // ── mask_key ────────────────────────────────────────────────────────

    #[test]
    fn test_mask_key_long_value_shows_prefix_and_suffix() {
        let key = "ctx7sk-abc123-def456-ghi789";
        assert_eq!(key.len(), 27, "Pré-condição: key deve ter 27 chars");
        let masked = mask_key(key);
        assert!(
            masked.starts_with("ctx7sk-abc12"),
            "Must start with the first 12 chars, got: {}",
            masked
        );
        assert!(
            masked.ends_with("i789"),
            "Must end with the last 4 chars, got: {}",
            masked
        );
        assert!(
            masked.contains("..."),
            "Must contain '...' between prefix and suffix, got: {}",
            masked
        );
    }

    #[test]
    fn test_mask_key_short_returns_asterisks() {
        let exactly_16_chars_key = "ctx7sk-abcdef012";
        assert_eq!(
            exactly_16_chars_key.len(),
            16,
            "Pré-condição: key deve ter 16 chars"
        );
        let masked = mask_key(exactly_16_chars_key);
        assert_eq!(
            masked, "***",
            "16-char key must return '***', got: {}",
            masked
        );
    }

    #[test]
    fn test_mask_key_empty_returns_asterisks() {
        let masked = mask_key("");
        assert_eq!(
            masked, "***",
            "Empty key must return '***', got: {}",
            masked
        );
    }

    #[test]
    fn test_mask_key_exactly_17_chars_masks_correctly() {
        let key = "ctx7sk-abcdef0123"; // 17 chars
        assert_eq!(key.len(), 17, "Pré-condição: key deve ter 17 chars");
        let masked = mask_key(key);
        assert!(
            masked.contains("..."),
            "Chave de 17 chars deve ser masked, obteve: {}",
            masked
        );
        assert_eq!(
            &masked[..12],
            &key[..12],
            "12-char prefix must be preserved"
        );
        assert!(
            masked.ends_with(&key[key.len() - 4..]),
            "4-char suffix must be preserved"
        );
    }

    // ── read_env_var_key ─────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_read_env_var_key_returns_some_when_set() {
        // SAFETY: tests serialised via #[serial_test::serial] guarantee absence of
        // concurrency. Required for compatibility with Rust 2024 edition.
        unsafe {
            std::env::set_var("CONTEXT7_API_KEYS", "ctx7sk-key-teste-01");
        }
        let result = read_env_var_key();
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
        }

        let keys = result.expect("Must return Some with valid key");
        assert_eq!(keys.len(), 1, "Must return exactly 1 key");
        assert_eq!(keys[0], "ctx7sk-key-teste-01");
    }

    #[test]
    #[serial_test::serial]
    fn test_read_env_var_key_accepts_multiple_comma_separated() {
        // SAFETY: idem
        unsafe {
            std::env::set_var(
                "CONTEXT7_API_KEYS",
                "ctx7sk-key-a, ctx7sk-key-b , ctx7sk-key-c",
            );
        }
        let result = read_env_var_key();
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
        }

        let keys = result.expect("Must return Some with multiple keys");
        assert_eq!(keys.len(), 3, "Must return 3 keys");
        assert_eq!(keys[0], "ctx7sk-key-a");
        assert_eq!(keys[1], "ctx7sk-key-b");
        assert_eq!(keys[2], "ctx7sk-key-c");
    }

    #[test]
    #[serial_test::serial]
    fn test_read_env_var_key_returns_none_when_empty() {
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_API_KEYS", "");
        }
        let result = read_env_var_key();
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
        }

        assert!(
            result.is_none(),
            "Must return None when env var is empty"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_read_env_var_key_returns_none_when_only_whitespace() {
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_API_KEYS", "   ,  ,  ");
        }
        let result = read_env_var_key();
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
        }

        assert!(
            result.is_none(),
            "Must return None when env var contains only whitespace/commas"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_read_env_var_key_returns_none_when_missing() {
        // SAFETY: idem
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
        }
        let result = read_env_var_key();

        assert!(
            result.is_none(),
            "Must return None when env var does not exist"
        );
    }

    // ── path traversal via CONTEXT7_HOME ──────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_context7_home_rejects_path_traversal() {
        let cases = ["../../../etc", "..", "/tmp/../etc"];
        for case in &cases {
            // SAFETY: env var manipulation in serial test context.
            unsafe {
                std::env::set_var("CONTEXT7_HOME", case);
            }
            let result = discover_config_path();
            unsafe {
                std::env::remove_var("CONTEXT7_HOME");
            }

            // Must fall back to XDG — the result must NOT contain ".."
            if let Some(path) = result {
                let s = path.to_string_lossy();
                assert!(
                    !s.contains(".."),
                    "Path traversal '{case}' não deve resultar em path com '..': {s}"
                );
            }
            // None também é aceitável (ProjectDirs ausente no ambiente de CI)
        }
    }

    // ── read_xdg_config via CONTEXT7_HOME ───────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_returns_none_for_nonexistent_file() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }
        let result = read_xdg_config();
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let value = result.expect("Must return Ok when file does not exist");
        assert!(
            value.is_none(),
            "Must return None when config.toml does not exist"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_reads_valid_toml_with_multiple_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        let toml_content = r#"schema_version = 1

[[keys]]
value = "ctx7sk-key-xdg-01"
added_at = "2026-01-01T00:00:00+00:00"

[[keys]]
value = "ctx7sk-key-xdg-02"
added_at = "2026-01-02T00:00:00+00:00"
"#;
        std::fs::write(context7_dir.join("config.toml"), toml_content)
            .expect("Must write config.toml");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }
        let result = read_xdg_config();
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let keys = result
            .expect("Must return Ok")
            .expect("Must return Some with keys");
        assert_eq!(keys.len(), 2, "Must return 2 keys");
        assert_eq!(keys[0], "ctx7sk-key-xdg-01");
        assert_eq!(keys[1], "ctx7sk-key-xdg-02");
    }

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_returns_err_on_invalid_toml() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        std::fs::write(
            context7_dir.join("config.toml"),
            "schema_version = INVALIDO\n[[[malformado",
        )
        .expect("Must write invalid TOML");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }
        let result = read_xdg_config();
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Must return Err when TOML is malformed"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_preserves_key_order() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        let toml_content = r#"schema_version = 1

[[keys]]
value = "ctx7sk-primeira"
added_at = "2026-01-01T00:00:00+00:00"

[[keys]]
value = "ctx7sk-segunda"
added_at = "2026-01-02T00:00:00+00:00"

[[keys]]
value = "ctx7sk-terceira"
added_at = "2026-01-03T00:00:00+00:00"
"#;
        std::fs::write(context7_dir.join("config.toml"), toml_content)
            .expect("Must write config.toml");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }
        let result = read_xdg_config();
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let keys = result
            .expect("Must return Ok")
            .expect("Must return Some");
        assert_eq!(keys[0], "ctx7sk-primeira");
        assert_eq!(keys[1], "ctx7sk-segunda");
        assert_eq!(keys[2], "ctx7sk-terceira");
    }

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_returns_none_for_empty_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        let toml_without_keys = "schema_version = 1\n";
        std::fs::write(context7_dir.join("config.toml"), toml_without_keys)
            .expect("Must write config.toml without keys");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }
        let result = read_xdg_config();
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let value = result.expect("Must return Ok");
        assert!(
            value.is_none(),
            "Must return None when config.toml exists but keys is empty"
        );
    }

    // ── write_xdg_config ───────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_write_xdg_config_roundtrip_serialises_and_deserialises() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let path =
            write_xdg_config("ctx7sk-roundtrip-01").expect("Must write config without error");

        let read_config = read_toml_config_from_path(&path)
            .expect("Must read TOML written by write_xdg_config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(read_config.schema_version, 1, "schema_version must be 1");
        assert_eq!(read_config.keys.len(), 1, "Must contain 1 key");
        assert_eq!(
            read_config.keys[0].value, "ctx7sk-roundtrip-01",
            "Valor da key deve ser preservado"
        );
        assert!(
            !read_config.keys[0].added_at.is_empty(),
            "added_at não deve ser vazio"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_write_xdg_config_creates_parent_dirs_if_missing() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let xdg_novo = temp_dir.path().join("xdg_inexistente");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", &xdg_novo);
        }

        let result = write_xdg_config("ctx7sk-mkdir-teste");
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let path = result.expect("Must create parent directory and write config");
        assert!(
            path.exists(),
            "Arquivo de config deve existir após escrita"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_write_xdg_config_does_not_duplicate_existing_key() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-unica").expect("First write must work");
        write_xdg_config("ctx7sk-unica").expect("Second write must not fail");

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(
            config.keys.len(),
            1,
            "Não deve duplicar key já existente — deve ter apenas 1"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_write_xdg_config_accumulates_distinct_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-key-a").expect("First write must work");
        write_xdg_config("ctx7sk-key-b").expect("Second write must work");
        write_xdg_config("ctx7sk-key-c").expect("Third write must work");

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(config.keys.len(), 3, "Must accumulate 3 distinct keys");
        let valores: Vec<&str> = config.keys.iter().map(|c| c.value.as_str()).collect();
        assert!(valores.contains(&"ctx7sk-key-a"));
        assert!(valores.contains(&"ctx7sk-key-b"));
        assert!(valores.contains(&"ctx7sk-key-c"));
    }

    #[test]
    #[cfg(unix)]
    #[serial_test::serial]
    fn test_write_xdg_config_applies_600_permissions_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let path =
            write_xdg_config("ctx7sk-perm-600").expect("Must write config without error");
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let metadata = std::fs::metadata(&path).expect("Must obtain file metadata");
        let mode = metadata.permissions().mode() & 0o777;

        assert_eq!(mode, 0o600, "Permissions must be 600, got: {:o}", mode);
    }

    // ── Serde TOML roundtrip ──────────────────────────────────────────────────

    #[test]
    fn test_file_config_serde_roundtrip_preserves_all_fields() {
        let original_config = FileConfig {
            schema_version: 1,
            keys: vec![
                StoredKey {
                    value: "ctx7sk-serde-01".to_string(),
                    added_at: "2026-01-01T12:00:00+00:00".to_string(),
                },
                StoredKey {
                    value: "ctx7sk-serde-02".to_string(),
                    added_at: "2026-01-02T12:00:00+00:00".to_string(),
                },
            ],
        };

        let toml_str = toml::to_string_pretty(&original_config)
            .expect("Must serialise FileConfig to TOML");
        let deserialised_config: FileConfig =
            toml::from_str(&toml_str).expect("Must deserialise TOML back to FileConfig");

        assert_eq!(
            deserialised_config.schema_version, original_config.schema_version,
            "schema_version must be preserved in roundtrip"
        );
        assert_eq!(
            deserialised_config.keys.len(),
            original_config.keys.len(),
            "Número de keys deve ser preservado"
        );
        assert_eq!(
            deserialised_config.keys[0].value, original_config.keys[0].value,
            "Valor da primeira key deve ser preservado"
        );
        assert_eq!(
            deserialised_config.keys[0].added_at, original_config.keys[0].added_at,
            "added_at da primeira key deve ser preservado"
        );
    }

    #[test]
    fn test_file_config_schema_version_always_present_in_serialisation() {
        let config = FileConfig {
            schema_version: 1,
            keys: Vec::new(),
        };

        let toml_str = toml::to_string_pretty(&config).expect("Must serialise to TOML");

        assert!(
            toml_str.contains("schema_version"),
            "schema_version must be present in TOML serialisation"
        );
        assert!(toml_str.contains('1'), "Value 1 must be present");
    }

    #[test]
    fn test_file_config_empty_keys_accepted_in_deserialisation() {
        let toml_str = "schema_version = 1\n";
        let config: FileConfig =
            toml::from_str(toml_str).expect("Must deserialise with keys absent (empty default)");

        assert_eq!(config.schema_version, 1);
        assert!(
            config.keys.is_empty(),
            "keys must be empty when not present in TOML"
        );
    }

    #[test]
    fn test_stored_key_preserves_added_at_as_utc_string() {
        let timestamp = "2026-04-08T20:00:00+00:00";
        let key = StoredKey {
            value: "ctx7sk-timestamp".to_string(),
            added_at: timestamp.to_string(),
        };

        let toml_str = toml::to_string_pretty(&key).expect("Must serialise StoredKey");
        let chave_de_volta: StoredKey =
            toml::from_str(&toml_str).expect("Must deserialise StoredKey");

        assert_eq!(
            chave_de_volta.added_at, timestamp,
            "Timestamp added_at deve ser preservado exactly"
        );
    }

    // ── load_api_keys (precedence) ─────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_load_api_keys_env_var_takes_priority_over_xdg() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        let toml_xdg = r#"schema_version = 1
[[keys]]
value = "ctx7sk-xdg-deve-ser-ignorada"
added_at = "2026-01-01T00:00:00+00:00"
"#;
        std::fs::write(context7_dir.join("config.toml"), toml_xdg)
            .expect("Must write XDG config");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_API_KEYS", "ctx7sk-env-var-prioritaria");
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = load_api_keys();

        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
            std::env::remove_var("CONTEXT7_HOME");
        }

        let keys = result.expect("Must load keys via env var");
        assert_eq!(keys.len(), 1);
        assert_eq!(
            keys[0], "ctx7sk-env-var-prioritaria",
            "Env var deve ter prioridade sobre XDG"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_load_api_keys_xdg_used_when_env_var_missing() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        let toml_xdg = r#"schema_version = 1
[[keys]]
value = "ctx7sk-via-xdg"
added_at = "2026-01-01T00:00:00+00:00"
"#;
        std::fs::write(context7_dir.join("config.toml"), toml_xdg)
            .expect("Must write XDG config");

        // SAFETY: idem
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = load_api_keys();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let keys = result.expect("Must load keys via XDG");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "ctx7sk-via-xdg");
    }

    #[test]
    #[serial_test::serial]
    fn test_load_api_keys_returns_err_when_nothing_available() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let empty_xdg_dir = temp_dir.path().join("xdg_vazio");
        std::fs::create_dir_all(&empty_xdg_dir).expect("Must create empty XDG directory");

        let no_env_dir = temp_dir.path().join("sem_env");
        std::fs::create_dir_all(&no_env_dir).expect("Must create directory without .env");

        // SAFETY: idem
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
            std::env::set_var("CONTEXT7_HOME", &empty_xdg_dir);
        }
        let original_cwd = std::env::current_dir().expect("Must obtain current CWD");
        std::env::set_current_dir(&no_env_dir).expect("Must change CWD");

        let result = load_api_keys();

        std::env::set_current_dir(&original_cwd).expect("Must restore CWD");
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Must return Err when no layer provides keys"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_read_env_cwd_reads_env_with_multiple_context7_api_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let conteudo_env = "CONTEXT7_API=ctx7sk-cwd-01\nCONTEXT7_API=ctx7sk-cwd-02\n";
        std::fs::write(temp_dir.path().join(".env"), conteudo_env)
            .expect("Must write temporary .env");

        let original_cwd = std::env::current_dir().expect("Must obtain CWD");
        std::env::set_current_dir(temp_dir.path()).expect("Must change CWD to temp");

        let result = read_env_cwd();

        std::env::set_current_dir(&original_cwd).expect("Must restore CWD");

        let keys = result.expect("Must return Some with keys from CWD .env");
        assert_eq!(keys.len(), 2, "Must read 2 keys from .env");
        assert_eq!(keys[0], "ctx7sk-cwd-01");
        assert_eq!(keys[1], "ctx7sk-cwd-02");
    }

    #[test]
    #[serial_test::serial]
    fn test_read_env_cwd_returns_none_when_env_missing() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");

        let original_cwd = std::env::current_dir().expect("Must obtain CWD");
        std::env::set_current_dir(temp_dir.path()).expect("Must change CWD to temp without .env");

        let result = read_env_cwd();

        std::env::set_current_dir(&original_cwd).expect("Must restore CWD");

        assert!(
            result.is_none(),
            "Must return None when there is no .env in CWD"
        );
    }

    #[test]
    fn test_discover_xdg_log_paths_returns_some_valid_path() {
        let result = discover_xdg_log_paths();

        if let Some(path) = result {
            let path_str = path.to_string_lossy();
            assert!(
                path_str.contains("context7"),
                "Caminho de logs XDG deve conter 'context7', obteve: {}",
                path_str
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn test_load_api_keys_env_cwd_used_when_env_var_and_xdg_missing() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let xdg_dir_no_config = temp_dir.path().join("xdg_sem_config");
        std::fs::create_dir_all(&xdg_dir_no_config).expect("Must create empty XDG directory");

        let cwd_dir = temp_dir.path().join("cwd_com_env");
        std::fs::create_dir_all(&cwd_dir).expect("Must create temporary CWD");
        std::fs::write(cwd_dir.join(".env"), "CONTEXT7_API=ctx7sk-cwd-camada-3\n")
            .expect("Must write .env in CWD");

        // SAFETY: idem
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
            std::env::set_var("CONTEXT7_HOME", &xdg_dir_no_config);
        }
        let original_cwd = std::env::current_dir().expect("Must obtain CWD");
        std::env::set_current_dir(&cwd_dir).expect("Must change CWD");

        let result = load_api_keys();

        std::env::set_current_dir(&original_cwd).expect("Must restore CWD");
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let keys = result.expect("Must load keys via CWD .env");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "ctx7sk-cwd-camada-3");
    }

    #[test]
    #[serial_test::serial]
    fn test_load_api_keys_falls_back_when_xdg_invalid() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        std::fs::write(context7_dir.join("config.toml"), "[[[invalido")
            .expect("Must write invalid TOML");

        let cwd_dir = temp_dir.path().join("cwd_fallback");
        std::fs::create_dir_all(&cwd_dir).expect("Must create CWD with .env");
        std::fs::write(cwd_dir.join(".env"), "CONTEXT7_API=ctx7sk-fallback-cwd\n")
            .expect("Must write .env in CWD");

        // SAFETY: idem
        unsafe {
            std::env::remove_var("CONTEXT7_API_KEYS");
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }
        let original_cwd = std::env::current_dir().expect("Must obtain CWD");
        std::env::set_current_dir(&cwd_dir).expect("Must change CWD");

        let result = load_api_keys();

        std::env::set_current_dir(&original_cwd).expect("Must restore CWD");
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let keys = result.expect("Must load keys via fallback CWD .env");
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "ctx7sk-fallback-cwd");
    }

    // ── cmd_keys_add ─────────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_add_creates_config_when_missing() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_add("ctx7sk-nova-key-add-test");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        result.expect("cmd_keys_add deve funcionar em config vazio");

        let path = temp_dir.path().join("context7").join("config.toml");
        assert!(
            path.exists(),
            "config.toml deve existir após cmd_keys_add"
        );

        let config = read_toml_config_from_path(&path).expect("Must read created config");
        assert_eq!(config.keys.len(), 1, "Config deve ter 1 key");
        assert_eq!(config.keys[0].value, "ctx7sk-nova-key-add-test");
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_add_accumulates_in_existing_config() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        cmd_keys_add("ctx7sk-key-um").expect("Primeira adição deve funcionar");
        cmd_keys_add("ctx7sk-key-dois").expect("Segunda adição deve funcionar");

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(config.keys.len(), 2, "Must accumulate 2 keys");
        assert_eq!(config.keys[0].value, "ctx7sk-key-um");
        assert_eq!(config.keys[1].value, "ctx7sk-key-dois");
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_add_does_not_duplicate_existing_key() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        cmd_keys_add("ctx7sk-unica-dedup").expect("Primeira adição deve funcionar");
        cmd_keys_add("ctx7sk-unica-dedup").expect("Segunda adição da mesma key não deve falhar");

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(config.keys.len(), 1, "Não deve duplicar key já existente");
    }

    #[test]
    #[cfg(unix)]
    #[serial_test::serial]
    fn test_cmd_keys_add_applies_600_permissions_on_unix() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        cmd_keys_add("ctx7sk-perm-600-keys-add").expect("Must add key without error");

        let path = discover_config_path().expect("Must have XDG path");
        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let metadata = std::fs::metadata(&path).expect("Must obtain metadata");
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "Permissões devem ser 600 após cmd_keys_add, obteve: {:o}",
            mode
        );
    }

    // ── cmd_keys_remove ───────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_remove_index_1_from_config_with_3_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-rem-alpha").expect("Must write key 1");
        write_xdg_config("ctx7sk-rem-beta").expect("Must write key 2");
        write_xdg_config("ctx7sk-rem-gamma").expect("Must write key 3");

        cmd_keys_remove(1).expect("Remove index 1 must work");

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(config.keys.len(), 2, "Must remain 2 keys after removal");
        assert_eq!(config.keys[0].value, "ctx7sk-rem-beta");
        assert_eq!(config.keys[1].value, "ctx7sk-rem-gamma");
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_remove_index_2_from_config_with_3_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-mid-alpha").expect("Must write key 1");
        write_xdg_config("ctx7sk-mid-beta").expect("Must write key 2");
        write_xdg_config("ctx7sk-mid-gamma").expect("Must write key 3");

        cmd_keys_remove(2).expect("Remove index 2 must work");

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(
            config.keys.len(),
            2,
            "Must remain 2 keys after removing the middle one"
        );
        assert_eq!(config.keys[0].value, "ctx7sk-mid-alpha");
        assert_eq!(config.keys[1].value, "ctx7sk-mid-gamma");
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_remove_index_zero_returns_err_with_message() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-idx-zero-test").expect("Must write key");

        let result = cmd_keys_remove(0);

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Invalid index 0 must return Err (exit code 1), got: {:?}",
            result
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_remove_index_greater_than_len_returns_err_with_message() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-overflow-test").expect("Must write key");

        let result = cmd_keys_remove(99);

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Index out of range must return Err (exit code 1), got: {:?}",
            result
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_remove_from_empty_config_returns_err() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_remove(1);

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Remove from empty config must return Err (exit code 1), got: {:?}",
            result
        );
    }

    // ── cmd_keys_clear ────────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_clear_with_yes_true_clears_all_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-clear-alpha").expect("Must write key 1");
        write_xdg_config("ctx7sk-clear-beta").expect("Must write key 2");

        let path = discover_config_path().expect("Must have XDG path");
        let before = read_toml_config_from_path(&path).expect("Must read before config");
        assert_eq!(before.keys.len(), 2, "Pré-condição: 2 keys before do clear");

        cmd_keys_clear(true).expect("clear with yes=true must work");

        let after = read_toml_config_from_path(&path).expect("Must read after config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            after.keys.is_empty(),
            "Após clear com yes=true, keys devem estar vazias"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_clear_with_yes_true_works_on_nonexistent_config() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_clear(true);

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_ok(),
            "clear on nonexistent config must return Ok (idempotent), got: {:?}",
            result
        );
    }

    // ── cmd_keys_import ───────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_import_valid_env_with_multiple_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let arquivo_env = temp_dir.path().join("keys.env");
        std::fs::write(
            &arquivo_env,
            "CONTEXT7_API=ctx7sk-import-alpha\nCONTEXT7_API=ctx7sk-import-beta\n",
        )
        .expect("Must write test .env");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_import(&arquivo_env);

        let path = discover_config_path().expect("Must have XDG path");
        let config = read_toml_config_from_path(&path).expect("Must read config after import");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        result.expect("valid .env import must work");
        assert_eq!(config.keys.len(), 2, "Must have imported 2 keys");

        let valores: Vec<&str> = config.keys.iter().map(|c| c.value.as_str()).collect();
        assert!(valores.contains(&"ctx7sk-import-alpha"));
        assert!(valores.contains(&"ctx7sk-import-beta"));
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_import_env_without_keys_returns_err() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let arquivo_env = temp_dir.path().join("vazio.env");
        std::fs::write(&arquivo_env, "# apenas comentario\nOUTRA_VAR=value\n")
            .expect("Must write .env without keys");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_import(&arquivo_env);

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Import de .env sem keys CONTEXT7_API deve retornar Err"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_import_nonexistent_file_returns_err() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let arquivo_inexistente = temp_dir.path().join("nao_existe.env");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_import(&arquivo_inexistente);

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_err(),
            "Import de file inexistente deve retornar Err"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_import_roundtrip_add_then_list() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let arquivo_env = temp_dir.path().join("roundtrip.env");
        std::fs::write(
            &arquivo_env,
            "CONTEXT7_API=ctx7sk-rtrip-01\nCONTEXT7_API=ctx7sk-rtrip-02\n",
        )
        .expect("Must write roundtrip .env");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        cmd_keys_import(&arquivo_env).expect("Import must work");

        let config = read_xdg_config_raw()
            .expect("Must return Ok")
            .expect("Must return Some after import");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert_eq!(
            config.keys.len(),
            2,
            "Roundtrip: deve ter 2 keys após import"
        );
        assert_eq!(config.keys[0].value, "ctx7sk-rtrip-01");
        assert_eq!(config.keys[1].value, "ctx7sk-rtrip-02");
    }

    // ── cmd_keys_export ───────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_export_empty_config_returns_ok() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_export();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_ok(),
            "Export of empty config must return Ok, got: {:?}",
            result
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_export_returns_ok_with_existing_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        write_xdg_config("ctx7sk-export-um").expect("Must write key 1");
        write_xdg_config("ctx7sk-export-dois").expect("Must write key 2");

        let result = cmd_keys_export();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            result.is_ok(),
            "Export com keys existentes deve retornar Ok, obteve: {:?}",
            result
        );
    }

    // ── read_xdg_config_raw ────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_raw_returns_none_without_file() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = read_xdg_config_raw();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let value = result.expect("Must return Ok");
        assert!(
            value.is_none(),
            "Must return None when config.toml does not exist"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_read_xdg_config_raw_returns_config_with_keys() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        let context7_dir = temp_dir.path().join("context7");
        std::fs::create_dir_all(&context7_dir).expect("Must create context7 directory");

        let toml = r#"schema_version = 1

[[keys]]
value = "ctx7sk-raw-01"
added_at = "2026-04-08T00:00:00+00:00"

[[keys]]
value = "ctx7sk-raw-02"
added_at = "2026-04-08T00:01:00+00:00"
"#;
        std::fs::write(context7_dir.join("config.toml"), toml).expect("Must write config.toml");

        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = read_xdg_config_raw();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let config = result
            .expect("Must return Ok")
            .expect("Must return Some with config");
        assert_eq!(config.keys.len(), 2);
        assert_eq!(config.keys[0].value, "ctx7sk-raw-01");
        assert_eq!(config.keys[1].value, "ctx7sk-raw-02");
    }

    #[test]
    #[serial_test::serial]
    fn test_cmd_keys_path_returns_ok() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let result = cmd_keys_path();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        result.expect("cmd_keys_path must return Ok");
    }

    #[test]
    #[serial_test::serial]
    fn test_discover_config_path_ends_with_config_toml() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        let path = discover_config_path();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let path = path.expect("Must return valid XDG path");
        assert!(
            path.to_string_lossy().ends_with("config.toml"),
            "Path must end with config.toml, got: {}",
            path.display()
        );
        assert!(
            path.to_string_lossy().contains("context7"),
            "Path must contain 'context7', got: {}",
            path.display()
        );
    }

    // ── fluxo completo ────────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_complete_flow_add_list_remove_clear() {
        let temp_dir = tempfile::TempDir::new().expect("Must create temporary directory");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", temp_dir.path());
        }

        cmd_keys_add("ctx7sk-fluxo-01").expect("Add 1 must work");
        cmd_keys_add("ctx7sk-fluxo-02").expect("Add 2 must work");
        cmd_keys_add("ctx7sk-fluxo-03").expect("Add 3 must work");

        let config_before = read_xdg_config_raw()
            .expect("Ok")
            .expect("Some com 3 keys");
        assert_eq!(config_before.keys.len(), 3, "Must have 3 keys after 3 adds");

        cmd_keys_remove(2).expect("Remove index 2 must work");

        let config_after_remove = read_xdg_config_raw()
            .expect("Ok")
            .expect("Some com 2 keys");
        assert_eq!(
            config_after_remove.keys.len(),
            2,
            "Must have 2 keys after remove"
        );
        assert_eq!(config_after_remove.keys[0].value, "ctx7sk-fluxo-01");
        assert_eq!(config_after_remove.keys[1].value, "ctx7sk-fluxo-03");

        cmd_keys_clear(true).expect("Clear com yes=true deve funcionar");

        let path = discover_config_path().expect("Must have path");
        let final_config = read_toml_config_from_path(&path).expect("Must read final config");

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        assert!(
            final_config.keys.is_empty(),
            "Após clear, keys devem estar vazias"
        );
    }

    // ── CONTEXT7_HOME override direto ─────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn test_context7_home_override_config_path() {
        let tmp = tempfile::TempDir::new().expect("Must create tempdir");
        // SAFETY: tests serialised via #[serial_test::serial] guarantee absence of
        // concurrency. Required for compatibility with Rust 2024 edition.
        unsafe {
            std::env::set_var("CONTEXT7_HOME", tmp.path());
        }

        let path = discover_config_path();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let path = path.expect("Must return Some when CONTEXT7_HOME is defined");
        let expected = tmp.path().join("context7").join("config.toml");
        assert_eq!(
            path, expected,
            "CONTEXT7_HOME must define path as {{CONTEXT7_HOME}}/context7/config.toml"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_context7_home_override_logs_path() {
        let tmp = tempfile::TempDir::new().expect("Must create tempdir");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", tmp.path());
        }

        let path = discover_xdg_log_paths();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        let path = path.expect("Must return Some when CONTEXT7_HOME is defined");
        let expected = tmp.path().join("context7").join("logs");
        assert_eq!(
            path, expected,
            "CONTEXT7_HOME must define logs as {{CONTEXT7_HOME}}/context7/logs"
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_context7_home_empty_falls_back_to_projectdirs() {
        let tmp = tempfile::TempDir::new().expect("Must create tempdir");
        // SAFETY: idem
        unsafe {
            std::env::set_var("CONTEXT7_HOME", "");
        }

        let path = discover_config_path();

        unsafe {
            std::env::remove_var("CONTEXT7_HOME");
        }

        // When CONTEXT7_HOME is empty, falls back to ProjectDirs — path must NOT be inside the tempdir
        if let Some(c) = path {
            let tmp_str = tmp.path().to_string_lossy();
            assert!(
                !c.to_string_lossy().starts_with(tmp_str.as_ref()),
                "CONTEXT7_HOME empty must not use the tempdir: {}",
                c.display()
            );
        }
        // If ProjectDirs returns None (CI without home), that is also acceptable
    }
}
