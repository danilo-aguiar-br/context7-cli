# How to use context7-cli / Como usar context7-cli

---

## English

`context7-cli` is a native Rust CLI that queries the [Context7](https://context7.com) REST API to search library documentation from your terminal. This guide covers installation on Linux, macOS, and Windows, plus complete usage examples and LLM integration patterns.

**Table of Contents (English)**

- [Step 1 — Download & Install](#step-1--download--install)
- [Step 2 — Get your Context7 API key](#step-2--get-your-context7-api-key)
- [Step 3 — Add the API key](#step-3--add-the-api-key)
- [Step 4 — Your first search](#step-4--your-first-search)
- [Step 5 — Fetch documentation](#step-5--fetch-documentation)
- [Step 6 — Language override](#step-6--language-override-v020)
- [Step 7 — Output formats](#step-7--output-formats)
- [Step 8 — Advanced: multi-key rotation](#step-8--advanced-multi-key-rotation)
- [Step 9 — Backup & restore](#step-9--backup--restore)
- [Step 10 — Shell completions](#step-10--shell-completions)
- [System prompt for LLMs (English)](#system-prompt-for-llms-english)
- [System prompt for LLMs (Portuguese)](#system-prompt-for-llms-portugu%C3%AAs)

---

### Step 1 — Download & Install

#### Linux

**Option A — Recommended: via `cargo install`**

```bash
# Requires Rust toolchain (https://rustup.rs)
cargo install context7-cli
```

**Option B — Build from source**

```bash
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli
cargo build --release
sudo cp target/release/context7 /usr/local/bin/context7
# or without sudo:
cp target/release/context7 ~/.local/bin/context7
```

**Verify installation:**

```bash
context7 --help
```

#### macOS

**Option A — Recommended: via `cargo install`**

```bash
# Install Rust from https://rustup.rs if needed
cargo install context7-cli
```

The binary is placed in `~/.cargo/bin/context7`. Ensure `~/.cargo/bin` is in your `PATH` (added automatically by rustup).

**Option B — Build from source**

```bash
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli
cargo build --release
cp target/release/context7 /usr/local/bin/context7
```

**Verify installation:**

```bash
context7 --help
```

#### Windows

**Option A — Recommended: via `cargo install` (PowerShell)**

```powershell
# 1. Install Rust from https://rustup.rs (64-bit installer)
# 2. Open a new PowerShell window, then:
cargo install context7-cli
```

The binary is placed at `%USERPROFILE%\.cargo\bin\context7.exe`. Rustup adds this to `PATH` automatically.

**Verify installation (PowerShell):**

```powershell
context7 --help
```

**Option B — Build from source (PowerShell)**

```powershell
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli
cargo build --release
# Binary at: target\release\context7.exe
# Copy to a directory in your PATH:
copy target\release\context7.exe $env:USERPROFILE\.cargo\bin\
```

**Option C — Download prebuilt binary**

Download the latest `context7-windows-x86_64.zip` from the [GitHub Releases page](https://github.com/danilo-aguiar-br/context7-cli/releases/latest), extract it, and copy `context7.exe` to a directory in your `PATH`.

```powershell
# Example using curl (PowerShell 7+)
curl -L -o context7.zip https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-windows-x86_64.zip
Expand-Archive context7.zip -DestinationPath .
copy context7.exe $env:USERPROFILE\.cargo\bin\
```

> **Note:** On Windows, `context7` automatically enables UTF-8 console mode (`SetConsoleOutputCP(65001)`) and ANSI color support at startup. Accented characters and colored output work correctly in both CMD and PowerShell without any extra configuration.

---

### Step 2 — Get your Context7 API key

1. Go to [https://context7.com](https://context7.com)
2. Sign up or log in
3. Navigate to your account settings or API section
4. Generate a new API key — it starts with `ctx7sk-`
5. Copy the full key (you will only see it once)

---

### Step 3 — Add the API key

```bash
context7 keys add ctx7sk-YOUR-KEY-HERE
```

> **Note:** Empty keys are rejected with exit code 1. Keys that do not start with `ctx7sk-` will trigger a non-blocking stderr warning (the key is still stored).

**Verify it was stored:**

```bash
context7 keys list
# Output:   [1]  ctx7sk-YOUR...HERE  (added: 2026-04-08 14:30:00)

context7 keys path
# Output: /home/you/.config/context7/config.toml

# JSON output for scripting
context7 keys list --json
# Output: [{"index":1,"masked_key":"ctx7sk-YOU...ERE","added_at":"2026-04-08 14:30:00"}]
```

**Storage location by OS:**

| OS | Path |
|----|------|
| Linux | `~/.config/context7/config.toml` |
| macOS | `~/Library/Application Support/context7/config.toml` |
| Windows | `%APPDATA%\context7\config\config.toml` |

Permissions: `600` on Unix (owner read/write only) — set automatically.

**Alternative — environment variable (CI/CD, no file needed):**

```bash
export CONTEXT7_API_KEYS="ctx7sk-key-01,ctx7sk-key-02"
context7 library react
```

**Supported environment variables:**

| Variable | Purpose |
|----------|---------|
| `CONTEXT7_API_KEYS` | Comma-separated API keys (overrides config file) |
| `CONTEXT7_LANG` | UI language: `en` or `pt` |
| `CONTEXT7_HOME` | Alternative XDG config directory (mainly for tests) |
| `RUST_LOG` | Log level: `error`, `warn`, `info`, `debug`, `trace` |

**Full key discovery hierarchy (precedence order):**

| Priority | Source | Format |
|----------|--------|--------|
| 1 (highest) | `CONTEXT7_API_KEYS` env var | Comma-separated keys |
| 2 | XDG config (`~/.config/context7/config.toml`) | TOML |
| 3 | `.env` file in current directory | `CONTEXT7_API=ctx7sk-...` |
| 4 (lowest) | Compile-time embed | `CONTEXT7_API_KEYS` at build time |

#### Using CONTEXT7_HOME for custom config locations

`CONTEXT7_HOME` overrides the XDG base directory used for the config file. Set it to any writable directory — `context7` will read and write `config.toml` there instead of the default path.

**Precedence:** `CONTEXT7_HOME` > `ProjectDirs` default (XDG on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows).

**Common use cases:**

```bash
# Dotfiles — keep context7 config in your dotfiles repo
export CONTEXT7_HOME="$XDG_CONFIG_HOME/dotfiles/context7"
context7 keys list

# NixOS / home-manager — declarative config path
export CONTEXT7_HOME="/etc/context7"

# Docker container — config in a mounted volume
# In Dockerfile or docker-compose.yml:
# ENV CONTEXT7_HOME=/app/state
CONTEXT7_HOME=/app/state context7 keys list

# Isolated tests — prevent real config from being read or modified
CONTEXT7_HOME=/tmp/context7-test-$$ context7 keys add ctx7sk-fake-key
```

> If `CONTEXT7_HOME` points to a non-existent directory, `context7` **creates the directory automatically** on first use. This makes the variable convenient for dotfiles setups — point it at your intended location and the necessary structure will be provisioned on demand. If `CONTEXT7_HOME` is empty or unset, `context7` falls back to XDG defaults (`$XDG_CONFIG_HOME/context7` or `~/.config/context7`).

---

### Step 4 — Your first search

```bash
context7 library react
```

Example output:

```
Libraries found:
────────────────────────────────────────────────────────────
1. React (trust 10.0/10)
   /reactjs/react.dev
   The library for web and native user interfaces

2. Preact (trust 8.1/10)
   /preactjs/preact
   Fast 3kB React alternative with the same modern API
```

**With semantic context for better ranking:**

```bash
context7 library react "effect hooks"
context7 library axum "middleware and routing"
context7 library tokio "async mpsc channel"
```

**Available aliases:** `lib`, `search`

```bash
context7 lib react
context7 search tokio
```

---

### Step 5 — Fetch documentation

Use the `id` from Step 4 (e.g. `/reactjs/react.dev`):

```bash
context7 docs /reactjs/react.dev --query "useEffect cleanup"
```

**Available aliases:** `doc`, `context`

```bash
context7 doc /rust-lang/rust --query "lifetimes and borrowing"
context7 context /tokio-rs/tokio --query "spawn_blocking"
```

**Flags for `docs`:**

| Flag | Description |
|------|-------------|
| `--query <TEXT>`, `-q <TEXT>` | Semantic search within the library docs |
| `--text` | Return plain text (no color, ideal for LLMs and pipes) |
| `--json` | Return structured JSON |

> `--text` and `--json` are mutually exclusive.

**Examples:**

```bash
# Human-readable (default)
context7 docs /reactjs/react.dev --query "useState"

# Plain text for LLM context
context7 docs /tokio-rs/tokio --query "spawn_blocking" --text

# JSON for scripting
context7 docs /tokio-rs/axum --json
```

---

### Step 6 — Language override

By default, `context7-cli` auto-detects your system language. You can override it at runtime:

```bash
# Force English output
context7 --lang en library react

# Force Portuguese output
context7 --lang pt docs /reactjs/react.dev --query "hooks"
```

**Permanent override via environment variable:**

```bash
# Set once in your shell profile (~/.bashrc, ~/.zshrc, etc.)
export CONTEXT7_LANG=en    # always English
export CONTEXT7_LANG=pt    # always Portuguese
```

**Auto-detect order:**

| Priority | Source | Example |
|----------|--------|---------|
| 1 (highest) | `--lang` CLI flag | `context7 --lang en library react` |
| 2 | `CONTEXT7_LANG` env var | `CONTEXT7_LANG=pt context7 ...` |
| 3 | System locale | `LANG=pt_BR.UTF-8` → Portuguese |
| 4 (default) | English fallback | (when locale is unrecognized) |

Any locale starting with `pt` (e.g., `pt_BR`, `pt_PT`) triggers Portuguese output automatically.

---

### Step 7 — Output formats

**Colored output (default):**

```
Documentation:
────────────────────────────────────────────────────────────
## useEffect

The useEffect hook lets you synchronize a component with an external system...

Sources:
  https://react.dev/reference/react/useEffect
```

**JSON output (`--json`):**

```json
{
  "snippets": [
    {
      "pageTitle": "useEffect",
      "codeTitle": "Basic useEffect example",
      "codeDescription": "Synchronize a component with an external system.",
      "codeLanguage": "javascript",
      "codeTokens": 42,
      "codeId": "https://react.dev/reference/react/useEffect",
      "codeList": [
        {
          "language": "javascript",
          "code": "useEffect(() => { /* ... */ }, [deps]);"
        }
      ],
      "relevance": 0.95,
      "model": "gemini-2.5-flash"
    }
  ]
}
```

**Library JSON output:**

```json
[
  {
    "id": "/reactjs/react.dev",
    "title": "React",
    "description": "A JavaScript library for building user interfaces",
    "trustScore": 9.8
  }
]
```

---

### Step 8 — Advanced: multi-key rotation

When you have multiple API keys, `context7-cli` rotates between them automatically to avoid rate limiting:

```bash
# Add multiple keys (one per command)
context7 keys add ctx7sk-primary-key
context7 keys add ctx7sk-secondary-key
context7 keys add ctx7sk-tertiary-key

context7 keys list
# Output:
#   [1]  ctx7sk-prim...key  (added: 2026-04-08 14:30:00)
#   [2]  ctx7sk-seco...key  (added: 2026-04-08 14:30:00)
#   [3]  ctx7sk-tert...key  (added: 2026-04-08 14:30:00)
```

**How rotation works:**

- Each request randomly picks a key from the pool (shuffle without replacement)
- If a request fails (401/403/429/5xx/network error), the next attempt uses a different key
- Up to 5 attempts total with exponential backoff: 500ms → 1s → 2s
- With 5+ keys, rate limiting is practically eliminated

**Remove a specific key:**

```bash
context7 keys remove 2    # removes key at index 2 (1-based)
```

**Key rotation is automatic:**

```bash
# Key rotation is automatic — every request shuffles keys randomly.
```

---

### Step 9 — Backup & restore

```bash
# Export all keys to .env format (plain text — protect this file!)
context7 keys export > ~/backup-context7.env

# Verify the export
cat ~/backup-context7.env
# Output: CONTEXT7_API=ctx7sk-full-value-here

# Wipe all stored keys
context7 keys clear --yes

# Restore from backup
context7 keys import ~/backup-context7.env
# Output: 3/3 key(s) imported successfully.

# Migrate from a legacy project .env file
context7 keys import /path/to/project/.env
```

---

### Troubleshooting

| Symptom | Cause | Solution |
|---------|-------|----------|
| `No API key found` | No key configured | Run `context7 keys add ctx7sk-...` |
| `401 Unauthorized` | Invalid or expired key | Check with `context7 keys list`, remove bad key, add a new one |
| `429 Too Many Requests` | Rate limit | The CLI retries automatically; add more keys for sustained workloads |
| `Network error / timeout` | No internet or slow connection | Check connectivity; CLI retries with backoff (30s timeout per attempt) |
| `command not found: context7` | Binary not in PATH | Ensure `~/.cargo/bin` (Linux/macOS) or `%USERPROFILE%\.cargo\bin` (Windows) is in PATH |
| `All API keys failed` | All keys are invalid | Run `context7 keys list`, remove invalid keys, add new ones from context7.com |
| `docs` returns "No valid API key after N attempts" | Schema mismatch or invalid keys | (1) Verify keys via `context7 keys list`; (2) confirm library ID via `context7 library <name>`; (3) upgrade to the latest version — use `cargo install context7-cli --force` |
| Accented chars garbled in CMD or PowerShell (e.g. `Nenhuma` shows as `Nenhuma`) | Legacy Windows code page (not UTF-8) | Fixed automatically in v0.2.4+ via `SetConsoleOutputCP(65001)` at startup. If you still see garbled output, ensure you are on v0.2.4+ (`context7 --version`). |

**View logs for detailed diagnostics:**

```bash
# Linux
bat -P ~/.local/state/context7/context7.log

# macOS
bat -P ~/Library/Application\ Support/context7/context7.log

# Dev mode (any OS)
bat -P logs/context7.log
```

**Increase log verbosity:**

```bash
RUST_LOG=context7=debug context7 library react
RUST_LOG=trace context7 docs /reactjs/react.dev --query "hooks"
```

---

### Integration examples

**With `jaq` (JSON parsing):**

```bash
# Get only library IDs
context7 library react --json | jaq '.[].id'

# Filter by trust score > 8
context7 library vue --json | jaq '[.[] | select(.trustScore > 8.0)]'

# Get the top result's ID
context7 library axum --json | jaq -r '.[0].id'
```

**With `rg` (search in documentation):**

```bash
# Search for a pattern in plain text docs
context7 docs /reactjs/react.dev --text | rg "useEffect"

# With 3-line context
context7 docs /rust-lang/rust --query "ownership" --text | rg -C 3 "borrow"
```

**Loop over results:**

```bash
for id in $(context7 library react --json | jaq -r '.[].id'); do
  echo "=== $id ==="
  context7 docs "$id" --query "getting started" --text
done
```

---

### Global flags (v0.5.0+)

These flags work with any subcommand and control output behavior globally:

| Flag | Description | Example |
|------|-------------|---------|
| `--no-color` | Disable all ANSI colors and decorations | `context7 --no-color library react` |
| `--plain` | Alias for `--no-color` | `context7 --plain docs /reactjs/react.dev --query "hooks"` |
| `--verbose`, `-v` | Enable debug-level logging to stderr | `context7 -v library react` |
| `--quiet` | Suppress all non-essential output | `context7 --quiet library react --json` |

```bash
# Disable colors for piping to a file
context7 --no-color library react > results.txt

# Debug a failing request
context7 -v docs /reactjs/react.dev --query "hooks" --text

# Quiet mode for scripts (only stdout data, no banners)
context7 --quiet library react --json | jaq '.[0].id'

# Plain output for LLM consumption
context7 --plain docs /tokio-rs/tokio --query "spawn" --text
```

---

### Exit codes (BSD sysexits, v0.5.0+)

`context7-cli` uses BSD-compatible exit codes for precise error handling in scripts:

| Exit code | Name | Meaning |
|-----------|------|---------|
| 0 | `EX_OK` | Success |
| 1 | `EX_GENERAL` | General runtime error |
| 2 | `EX_USAGE` | Invalid CLI arguments or flags |
| 65 | `EX_DATAERR` | Input data error (malformed query, invalid JSON) |
| 66 | `EX_NOINPUT` | Library not found or invalid library ID |
| 69 | `EX_UNAVAILABLE` | Service unavailable (API down, all retries failed) |
| 74 | `EX_IOERR` | I/O error (config file read/write failure) |
| 77 | `EX_NOPERM` | Invalid or missing API keys |
| 130 | `EX_SIGINT` | Interrupted by Ctrl+C (SIGINT on Unix, CTRL_C_EVENT on Windows) |

```bash
# Handle exit codes in scripts
context7 library react --json
case $? in
  0)   echo "Success" ;;
  66)  echo "Library not found — check the ID" ;;
  69)  echo "Service unavailable — retry later" ;;
  77)  echo "Invalid API keys — run: context7 keys add ctx7sk-..." ;;
  130) echo "Interrupted by user" ;;
  *)   echo "Unexpected error: $?" ;;
esac
```

---

### Color control via environment variables (v0.5.0+)

`context7-cli` respects the [NO_COLOR](https://no-color.org/) standard and `CLICOLOR_FORCE`:

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | When set (any value), disables all ANSI colors — equivalent to `--no-color` |
| `CLICOLOR_FORCE` | When set to `1`, forces colors even when stdout is not a TTY |

```bash
# Disable colors globally (all runs in this shell session)
export NO_COLOR=1
context7 library react

# Force colors in a pipe (e.g., for `less -R`)
CLICOLOR_FORCE=1 context7 docs /reactjs/react.dev --query "hooks" | less -R

# One-shot: disable colors for a single run
NO_COLOR=1 context7 library react --json
```

---

### NDJSON output format (v0.5.0+)

When `--json` is used, `context7-cli` emits Newline-Delimited JSON (NDJSON) — each line is a self-contained JSON object with a `type` field and `timestamp`:

```bash
context7 library react --json
# Each line is a separate JSON object:
# {"type":"library","timestamp":"2026-04-16T12:00:00Z","data":{...}}
```

Parse with `jaq`:

```bash
# Extract all library IDs
context7 library react --json | jaq -r 'select(.type == "library") | .data.id'

# Filter by type
context7 docs /reactjs/react.dev --query "hooks" --json | jaq 'select(.type == "snippet")'
```

---

### Signal handling (v0.5.0+)

- Ctrl+C sends SIGINT on Unix and CTRL_C_EVENT on Windows
- `context7-cli` catches the signal, performs cleanup, and exits with code 130
- In-flight HTTP requests are cancelled gracefully
- No partial output is written to stdout after interruption

---

### ASCII fallback (v0.5.0+)

When stdout is not a TTY (piped or redirected) or `NO_COLOR` is set, `context7-cli` replaces Unicode decorations with ASCII equivalents:

| Unicode | ASCII | Context |
|---------|-------|---------|
| `\u25b8` | `>` | Bullet points |
| `\u2500` | `-` | Horizontal rules |

This ensures clean output in terminals without Unicode support and in CI/CD logs.

---

### ARM Linux targets (v0.5.0+)

Pre-built binaries are available for ARM64 Linux:

| Target | Use case |
|--------|----------|
| `aarch64-unknown-linux-gnu` | Raspberry Pi OS, Ubuntu on ARM, AWS Graviton |
| `aarch64-unknown-linux-musl` | Alpine Linux on ARM, minimal containers |

```bash
# Download ARM64 glibc binary
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-aarch64-unknown-linux-gnu.tar.gz \
  | tar xz
chmod +x context7 && mv context7 /usr/local/bin/

# Download ARM64 musl static binary (Alpine, containers)
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-aarch64-unknown-linux-musl.tar.gz \
  | tar xz
chmod +x context7 && mv context7 /usr/local/bin/
```

---

### Step 10 — Shell completions

Enable tab-completion for `context7` in your shell. Install once — works forever.

#### Bash

```bash
# Install for current user
context7 completions bash > ~/.local/share/bash-completion/completions/context7

# Reload (or open a new terminal)
source ~/.bashrc
```

#### Zsh

```bash
# Add ~/.zfunc to your fpath (add to ~/.zshrc if not already present):
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit

mkdir -p ~/.zfunc
context7 completions zsh > ~/.zfunc/_context7

# Reload completions
compinit
```

#### Fish

```bash
context7 completions fish > ~/.config/fish/completions/context7.fish
# Completions load automatically on next shell start
```

#### PowerShell

```powershell
# Add to your $PROFILE (runs on every shell start)
context7 completions powershell >> $PROFILE

# Reload profile
. $PROFILE
```

#### Elvish

```bash
context7 completions elvish > ~/.config/elvish/lib/context7.elv
# Import in ~/.config/elvish/rc.elv:
# use context7
```

---

### System prompt for LLMs (English)

Copy-paste this into your LLM's system prompt to enable automatic documentation fetching:

```
You have access to the `context7` CLI to fetch up-to-date technical documentation for libraries and frameworks.

## Available tool: `context7`

### When to use
- When you need documentation for any library (React, Vue, Axum, Tokio, etc.)
- When the user asks about APIs, configuration, or usage of a specific library
- When your training data may be outdated for a particular library
- Before suggesting code that depends on external APIs

### How to use: three subcommands

#### Subcommand `library` — Discover a library's ID
```bash
context7 library <NAME> [OPTIONAL_CONTEXT]
```
- `NAME`: library name (e.g., `react`, `axum`, `tokio`, `vue`)
- `OPTIONAL_CONTEXT`: text to refine ranking (e.g., `"effect hooks"`)
- Returns: list of libraries with IDs, titles, and trust scores
- **Always use this subcommand first to get the correct ID**

Examples:
```bash
context7 library react "effect hooks"
context7 library axum "middleware and routing"
context7 library tokio "async channels"
```

#### Subcommand `docs` — Fetch documentation by ID
```bash
context7 docs <LIBRARY_ID> [--query <TEXT>] [--text]
```
- `LIBRARY_ID`: ID in `/org/repo` format obtained via `library` (e.g., `/reactjs/react.dev`)
- `--query <TEXT>`: specific topic to search (optional but recommended)
- `--text`: returns plain text (recommended for inserting into context)
- `--json`: returns structured JSON (incompatible with `--text`)

Examples:
```bash
context7 docs /reactjs/react.dev --query "useEffect and cleanup" --text
context7 docs /tokio-rs/tokio --query "spawn_blocking" --text
context7 docs /rust-lang/rust --query "lifetimes and borrowing" --text
```

### Correct 2-step workflow

**Step 1:** Discover the ID
```bash
context7 library react "effect hooks"
# → Returns: /reactjs/react.dev, /preactjs/preact, ...
```

**Step 2:** Fetch documentation with the correct ID
```bash
context7 docs /reactjs/react.dev --query "useEffect dependencies" --text
```

### Important rules
- **NEVER guess a LIBRARY_ID** — always use `library` first to obtain it
- `--text` and `--json` are mutually exclusive: use only one
- The `--json` flag is global and works with any subcommand
- If `library` returns no results, try name variations (e.g., `react` vs `reactjs`)
- 401/403 errors indicate a problem with the API key (check with `context7 keys list`)
- 429 errors indicate rate limiting: the CLI retries automatically

### Prerequisites
- API key configured via `context7 keys add <KEY>` (recommended)
- `context7` binary available in PATH
```

---

### System prompt for LLMs (Portuguese)

See [Passo 9 abaixo (Português)](#system-prompt-para-llms-portugu%C3%AAs) for the Portuguese version.

---

## Português

A CLI `context7-cli` é um binário Rust nativo que consulta a API REST pública do [Context7](https://context7.com) (`https://context7.com/api/v1`) para buscar documentação de bibliotecas diretamente no terminal. Este guia cobre instalação no Linux, macOS e Windows, além de exemplos completos de uso e padrões de integração com LLMs.

**Sumário (Português)**

- [Passo 1 — Download e Instalação](#passo-1--download-e-instalação)
- [Passo 2 — Obter sua chave de API do Context7](#passo-2--obter-sua-chave-de-api-do-context7)
- [Passo 3 — Adicionar a chave de API](#passo-3--adicionar-a-chave-de-api)
- [Passo 4 — Primeira busca](#passo-4--primeira-busca)
- [Passo 5 — Buscar documentação](#passo-5--buscar-documentação)
- [Passo 6 — Override de idioma](#passo-6--override-de-idioma-v020)
- [Passo 7 — Formatos de saída](#passo-7--formatos-de-saída)
- [Passo 8 — Avançado: rotação de múltiplas chaves](#passo-8--avançado-rotação-de-múltiplas-chaves)
- [Passo 9 — Backup e restauração](#passo-9--backup-e-restauração)
- [Passo 10 — Autocompletar no Shell](#passo-10--autocompletar-no-shell)
- [System prompt para LLMs (Português)](#system-prompt-para-llms-portugu%C3%AAs)

---

### Passo 1 — Download e Instalação

#### Linux

**Opção A — Recomendada: via `cargo install`**

```bash
# Requer Rust toolchain (https://rustup.rs)
cargo install context7-cli
```

**Opção B — Build a partir do fonte**

```bash
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli
cargo build --release
sudo cp target/release/context7 /usr/local/bin/context7
# ou sem sudo:
cp target/release/context7 ~/.local/bin/context7
```

**Verificar instalação:**

```bash
context7 --help
```

#### macOS

**Opção A — Recomendada: via `cargo install`**

```bash
# Instale o Rust em https://rustup.rs se necessário
cargo install context7-cli
```

O binário é colocado em `~/.cargo/bin/context7`. O rustup adiciona esse diretório ao `PATH` automaticamente.

**Opção B — Build a partir do fonte**

```bash
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli
cargo build --release
cp target/release/context7 /usr/local/bin/context7
```

**Verificar instalação:**

```bash
context7 --help
```

#### Windows

**Opção A — Recomendada: via `cargo install` (PowerShell)**

```powershell
# 1. Instale o Rust em https://rustup.rs (instalador 64-bit)
# 2. Abra um novo PowerShell e execute:
cargo install context7-cli
```

O binário é colocado em `%USERPROFILE%\.cargo\bin\context7.exe`. O rustup adiciona esse diretório ao `PATH` automaticamente.

**Verificar instalação (PowerShell):**

```powershell
context7 --help
```

**Opção B — Build a partir do fonte (PowerShell)**

```powershell
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli
cargo build --release
# Binário em: target\release\context7.exe
copy target\release\context7.exe $env:USERPROFILE\.cargo\bin\
```

**Opção C — Download do binário pré-compilado**

Baixe o arquivo `context7-windows-x86_64.zip` mais recente na [página de Releases do GitHub](https://github.com/danilo-aguiar-br/context7-cli/releases/latest), extraia e copie `context7.exe` para um diretório no seu `PATH`.

```powershell
# Exemplo com curl (PowerShell 7+)
curl -L -o context7.zip https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-windows-x86_64.zip
Expand-Archive context7.zip -DestinationPath .
copy context7.exe $env:USERPROFILE\.cargo\bin\
```

> **Nota:** No Windows, o `context7` habilita automaticamente o modo UTF-8 no console (`SetConsoleOutputCP(65001)`) e suporte a cores ANSI na inicialização. Caracteres acentuados e saída colorida funcionam corretamente tanto no CMD quanto no PowerShell, sem configuração adicional.

---

### Passo 2 — Obter sua chave de API do Context7

1. Acesse [https://context7.com](https://context7.com)
2. Cadastre-se ou faça login
3. Navegue até as configurações da conta ou seção de API
4. Gere uma nova chave de API — ela começa com `ctx7sk-`
5. Copie a chave completa (você só a verá uma vez)

---

### Passo 3 — Adicionar a chave de API

```bash
context7 keys add ctx7sk-SUA-CHAVE-AQUI
```

> **Nota:** Chaves vazias são rejeitadas com exit code 1. Chaves que não começam com `ctx7sk-` exibem um aviso não-bloqueante no stderr (a chave ainda é armazenada).

**Verificar se foi salva:**

```bash
context7 keys list
# Saída:   [1]  ctx7sk-SUA...QUI  (added: 2026-04-08 14:30:00)

context7 keys path
# Saída: /home/usuario/.config/context7/config.toml

# Saída JSON para automação
context7 keys list --json
# Saída: [{"index":1,"masked_key":"ctx7sk-SUA...QUI","added_at":"2026-04-08 14:30:00"}]
```

**Localização do arquivo de configuração por OS:**

| OS | Caminho |
|----|---------|
| Linux | `~/.config/context7/config.toml` |
| macOS | `~/Library/Application Support/context7/config.toml` |
| Windows | `%APPDATA%\context7\config\config.toml` |

Permissões: `600` no Unix (leitura/escrita apenas do proprietário) — configuradas automaticamente.

**Alternativa — variável de ambiente (CI/CD, sem arquivo):**

```bash
export CONTEXT7_API_KEYS="ctx7sk-chave-01,ctx7sk-chave-02"
context7 library react
```

**Variáveis de ambiente suportadas:**

| Variável | Finalidade |
|----------|-----------|
| `CONTEXT7_API_KEYS` | Chaves de API separadas por vírgula (sobrepõe o arquivo de config) |
| `CONTEXT7_LANG` | Idioma da interface: `en` ou `pt` |
| `CONTEXT7_HOME` | Diretório XDG alternativo (principalmente para testes) |
| `RUST_LOG` | Nível de log: `error`, `warn`, `info`, `debug`, `trace` |

**Hierarquia completa de descoberta de chaves (ordem de precedência):**

| Prioridade | Fonte | Formato |
|------------|-------|---------|
| 1 (maior) | Variável de ambiente `CONTEXT7_API_KEYS` | Chaves separadas por vírgula |
| 2 | Config XDG (`~/.config/context7/config.toml`) | TOML |
| 3 | Arquivo `.env` no diretório atual | `CONTEXT7_API=ctx7sk-...` |
| 4 (menor) | Embutida em compile-time | `CONTEXT7_API_KEYS` no build |

#### Usando CONTEXT7_HOME para locais customizados

`CONTEXT7_HOME` sobrepõe o diretório base XDG usado para o arquivo de configuração. Defina-o como qualquer diretório com permissão de escrita — o `context7` lerá e gravará o `config.toml` nesse local em vez do caminho padrão.

**Precedência:** `CONTEXT7_HOME` > padrão `ProjectDirs` (XDG no Linux, `~/Library/Application Support` no macOS, `%APPDATA%` no Windows).

**Casos de uso comuns:**

```bash
# Dotfiles — manter a config do context7 no repositório de dotfiles
export CONTEXT7_HOME="$XDG_CONFIG_HOME/dotfiles/context7"
context7 keys list

# NixOS / home-manager — caminho declarativo de config
export CONTEXT7_HOME="/etc/context7"

# Contêiner Docker — config em volume montado
# No Dockerfile ou docker-compose.yml:
# ENV CONTEXT7_HOME=/app/state
CONTEXT7_HOME=/app/state context7 keys list

# Testes isolados — evitar que a config real seja lida ou modificada
CONTEXT7_HOME=/tmp/context7-teste-$$ context7 keys add ctx7sk-chave-falsa
```

> Se `CONTEXT7_HOME` apontar para um diretório inexistente, o `context7` **cria o diretório automaticamente** no primeiro uso. Isso torna a variável conveniente para setups de dotfiles — aponte para o local desejado e a estrutura necessária será provisionada sob demanda. Se `CONTEXT7_HOME` estiver vazia ou não definida, o `context7` usa os padrões XDG (`$XDG_CONFIG_HOME/context7` ou `~/.config/context7`).

---

### Passo 4 — Primeira busca

```bash
context7 library react
```

Exemplo de saída:

```
Bibliotecas encontradas:
────────────────────────────────────────────────────────────
1. React (confiança 10.0/10)
   /reactjs/react.dev
   The library for web and native user interfaces

2. Preact (confiança 8.1/10)
   /preactjs/preact
   Fast 3kB React alternative with the same modern API
```

**Com contexto semântico para ranking mais preciso:**

```bash
context7 library react "hooks de efeito"
context7 library axum "middleware e rotas"
context7 library tokio "canal mpsc assíncrono"
```

**Aliases disponíveis:** `lib`, `search`

```bash
context7 lib react
context7 search tokio
```

---

### Passo 5 — Buscar documentação

Use o `id` do Passo 4 (ex: `/reactjs/react.dev`):

```bash
context7 docs /reactjs/react.dev --query "useEffect e cleanup"
```

**Aliases disponíveis:** `doc`, `context`

```bash
context7 doc /rust-lang/rust --query "lifetimes e borrowing"
context7 context /tokio-rs/tokio --query "spawn_blocking"
```

**Flags para `docs`:**

| Flag | Descrição |
|------|-----------|
| `--query <TEXTO>`, `-q <TEXTO>` | Busca semântica dentro da documentação da biblioteca |
| `--text` | Retorna texto plano (sem cor, ideal para LLMs e pipes) |
| `--json` | Retorna JSON estruturado |

> `--text` e `--json` são mutuamente exclusivos.

**Exemplos:**

```bash
# Saída legível para humanos (padrão)
context7 docs /reactjs/react.dev --query "useState"

# Texto plano para contexto de LLM
context7 docs /tokio-rs/tokio --query "spawn_blocking" --text

# JSON para scripting
context7 docs /tokio-rs/axum --json
```

---

### Passo 6 — Override de idioma

Por padrão, o `context7-cli` detecta automaticamente o idioma do sistema. Você pode sobrescrever em runtime:

```bash
# Forçar saída em inglês
context7 --lang en library react

# Forçar saída em português
context7 --lang pt docs /reactjs/react.dev --query "hooks"
```

**Override permanente via variável de ambiente:**

```bash
# Configure uma vez no seu shell profile (~/.bashrc, ~/.zshrc, etc.)
export CONTEXT7_LANG=pt    # sempre português
export CONTEXT7_LANG=en    # sempre inglês
```

**Ordem de detecção automática:**

| Prioridade | Fonte | Exemplo |
|------------|-------|---------|
| 1 (maior) | Flag `--lang` na CLI | `context7 --lang pt library react` |
| 2 | Variável de ambiente `CONTEXT7_LANG` | `CONTEXT7_LANG=pt context7 ...` |
| 3 | Locale do sistema | `LANG=pt_BR.UTF-8` → português |
| 4 (padrão) | Fallback inglês | (quando locale não reconhecido) |

Qualquer locale começando com `pt` (ex: `pt_BR`, `pt_PT`) ativa o português automaticamente.

---

### Passo 7 — Formatos de saída

**Saída colorida (padrão):**

```
Documentação:
────────────────────────────────────────────────────────────
## useEffect

O hook useEffect permite sincronizar um componente com um sistema externo...

Fontes:
  https://react.dev/reference/react/useEffect
```

**Saída JSON (`--json`):**

```json
{
  "snippets": [
    {
      "pageTitle": "useEffect",
      "codeTitle": "Exemplo básico de useEffect",
      "codeDescription": "Sincronize um componente com um sistema externo.",
      "codeLanguage": "javascript",
      "codeTokens": 42,
      "codeId": "https://react.dev/reference/react/useEffect",
      "codeList": [
        {
          "language": "javascript",
          "code": "useEffect(() => { /* ... */ }, [deps]);"
        }
      ],
      "relevance": 0.95,
      "model": "gemini-2.5-flash"
    }
  ]
}
```

**JSON de library:**

```json
[
  {
    "id": "/reactjs/react.dev",
    "title": "React",
    "description": "A JavaScript library for building user interfaces",
    "trustScore": 9.8
  }
]
```

---

### Passo 8 — Avançado: rotação de múltiplas chaves

Com múltiplas chaves, a `context7-cli` alterna entre elas automaticamente para evitar rate limiting:

```bash
# Adicionar múltiplas chaves (uma por comando)
context7 keys add ctx7sk-chave-principal
context7 keys add ctx7sk-chave-secundaria
context7 keys add ctx7sk-chave-terciaria

context7 keys list
# Saída:
#   [1]  ctx7sk-chav...pal  (added: 2026-04-08 14:30:00)
#   [2]  ctx7sk-chav...ria  (added: 2026-04-08 14:30:00)
#   [3]  ctx7sk-chav...ria  (added: 2026-04-08 14:30:00)
```

**Como funciona a rotação:**

- Cada requisição escolhe aleatoriamente uma chave do pool (shuffle sem reposição)
- Se uma requisição falhar (401/403/429/5xx/erro de rede), a próxima tentativa usa uma chave diferente
- Até 5 tentativas com backoff exponencial: 500ms → 1s → 2s
- Com 5+ chaves, o rate limiting é praticamente eliminado

**Remover chave específica:**

```bash
context7 keys remove 2    # remove a chave no índice 2 (1-based)
```

**A rotação de chaves é automática:**

```bash
# A rotação de chaves é automática — cada requisição embaralha as chaves aleatoriamente.
```

---

### Passo 9 — Backup e restauração

```bash
# Exportar todas as chaves em formato .env (texto claro — proteja este arquivo!)
context7 keys export > ~/backup-context7.env

# Verificar o export
cat ~/backup-context7.env
# Saída: CONTEXT7_API=ctx7sk-valor-completo-aqui

# Apagar todas as chaves armazenadas
context7 keys clear --yes

# Restaurar a partir do backup
context7 keys import ~/backup-context7.env
# Saída: 3/3 chave(s) importada(s) com sucesso.

# Migrar a partir de arquivo .env legado de um projeto
context7 keys import /caminho/do/projeto/.env
```

---

### Solução de problemas

| Sintoma | Causa | Solução |
|---------|-------|---------|
| `Nenhuma chave de API encontrada` | Nenhuma chave configurada | Execute `context7 keys add ctx7sk-...` |
| `401 Unauthorized` | Chave inválida ou expirada | Verifique com `context7 keys list`, remova a chave inválida e adicione uma nova |
| `429 Too Many Requests` | Rate limit atingido | A CLI faz retry automaticamente; adicione mais chaves para cargas contínuas |
| Erro de rede / timeout | Sem internet ou conexão lenta | Verifique conectividade; CLI faz retry com backoff (timeout de 30s por tentativa) |
| `command not found: context7` | Binário não está no PATH | Certifique-se de que `~/.cargo/bin` (Linux/macOS) ou `%USERPROFILE%\.cargo\bin` (Windows) está no PATH |
| `Todas as chaves de API falharam` | Todas as chaves são inválidas | Execute `context7 keys list`, remova as inválidas e adicione novas em context7.com |
| `docs` retorna "Nenhuma chave de API válida após N tentativas" | Schema incompatível ou chaves inválidas | (1) Verifique as chaves via `context7 keys list`; (2) confirme o library ID via `context7 library <nome>`; (3) atualize para a versão mais recente — use `cargo install context7-cli --force` |
| Caracteres acentuados quebrados no CMD ou PowerShell (ex.: `ção` aparece como `??o`) | Code page legado do Windows (não UTF-8) | Corrigido automaticamente na v0.2.4+ via `SetConsoleOutputCP(65001)` na inicialização. Se ainda ocorrer, verifique se está na v0.2.4+ (`context7 --version`). |

**Ver logs para diagnóstico detalhado:**

```bash
# Linux
bat -P ~/.local/state/context7/context7.log

# macOS
bat -P ~/Library/Application\ Support/context7/context7.log

# Modo dev (qualquer OS)
bat -P logs/context7.log
```

**Aumentar verbosidade dos logs:**

```bash
RUST_LOG=context7=debug context7 library react
RUST_LOG=trace context7 docs /reactjs/react.dev --query "hooks"
```

---

### Exemplos de integração

**Com `jaq` (parsing JSON):**

```bash
# Obter apenas os IDs das bibliotecas
context7 library react --json | jaq '.[].id'

# Filtrar por trust score > 8
context7 library vue --json | jaq '[.[] | select(.trustScore > 8.0)]'

# Obter o ID do primeiro resultado
context7 library axum --json | jaq -r '.[0].id'
```

**Com `rg` (busca na documentação):**

```bash
# Buscar padrão em documentação de texto plano
context7 docs /reactjs/react.dev --text | rg "useEffect"

# Com contexto de 3 linhas
context7 docs /rust-lang/rust --query "ownership" --text | rg -C 3 "borrow"
```

**Loop sobre resultados:**

```bash
for id in $(context7 library react --json | jaq -r '.[].id'); do
  echo "=== $id ==="
  context7 docs "$id" --query "getting started" --text
done
```

---

### Flags globais (v0.5.0+)

Estas flags funcionam com qualquer subcomando e controlam o comportamento da saida globalmente:

| Flag | Descricao | Exemplo |
|------|-----------|---------|
| `--no-color` | Desabilita todas as cores e decoracoes ANSI | `context7 --no-color library react` |
| `--plain` | Alias para `--no-color` | `context7 --plain docs /reactjs/react.dev --query "hooks"` |
| `--verbose`, `-v` | Habilita logging nivel debug no stderr | `context7 -v library react` |
| `--quiet` | Suprime toda saida nao-essencial | `context7 --quiet library react --json` |

```bash
# Desabilitar cores para redirecionar a arquivo
context7 --no-color library react > resultados.txt

# Depurar uma requisicao com falha
context7 -v docs /reactjs/react.dev --query "hooks" --text

# Modo silencioso para scripts (apenas dados no stdout, sem banners)
context7 --quiet library react --json | jaq '.[0].id'

# Saida sem formatacao para consumo por LLM
context7 --plain docs /tokio-rs/tokio --query "spawn" --text
```

---

### Codigos de saida (BSD sysexits, v0.5.0+)

A `context7-cli` usa codigos de saida compativeis com BSD para tratamento preciso de erros em scripts:

| Codigo de saida | Nome | Significado |
|-----------------|------|-------------|
| 0 | `EX_OK` | Sucesso |
| 1 | `EX_GENERAL` | Erro geral de runtime |
| 2 | `EX_USAGE` | Argumentos ou flags de CLI invalidos |
| 65 | `EX_DATAERR` | Erro nos dados de entrada (query malformada, JSON invalido) |
| 66 | `EX_NOINPUT` | Biblioteca nao encontrada ou ID de biblioteca invalido |
| 69 | `EX_UNAVAILABLE` | Servico indisponivel (API fora do ar, todas as tentativas falharam) |
| 74 | `EX_IOERR` | Erro de I/O (falha na leitura/escrita do arquivo de config) |
| 77 | `EX_NOPERM` | Chaves de API invalidas ou ausentes |
| 130 | `EX_SIGINT` | Interrompido por Ctrl+C (SIGINT no Unix, CTRL_C_EVENT no Windows) |

```bash
# Tratar codigos de saida em scripts
context7 library react --json
case $? in
  0)   echo "Sucesso" ;;
  66)  echo "Biblioteca nao encontrada — verifique o ID" ;;
  69)  echo "Servico indisponivel — tente novamente mais tarde" ;;
  77)  echo "Chaves de API invalidas — execute: context7 keys add ctx7sk-..." ;;
  130) echo "Interrompido pelo usuario" ;;
  *)   echo "Erro inesperado: $?" ;;
esac
```

---

### Controle de cores via variaveis de ambiente (v0.5.0+)

A `context7-cli` respeita o padrao [NO_COLOR](https://no-color.org/) e a variavel `CLICOLOR_FORCE`:

| Variavel | Efeito |
|----------|--------|
| `NO_COLOR` | Quando definida (qualquer valor), desabilita todas as cores ANSI — equivalente a `--no-color` |
| `CLICOLOR_FORCE` | Quando definida como `1`, forca cores mesmo quando stdout nao e TTY |

```bash
# Desabilitar cores globalmente (todas as execucoes nesta sessao do shell)
export NO_COLOR=1
context7 library react

# Forcar cores em um pipe (ex: para `less -R`)
CLICOLOR_FORCE=1 context7 docs /reactjs/react.dev --query "hooks" | less -R

# Execucao unica: desabilitar cores para uma unica execucao
NO_COLOR=1 context7 library react --json
```

---

### Formato de saida NDJSON (v0.5.0+)

Quando `--json` e usado, a `context7-cli` emite Newline-Delimited JSON (NDJSON) — cada linha e um objeto JSON autonomo com campo `type` e `timestamp`:

```bash
context7 library react --json
# Cada linha e um objeto JSON separado:
# {"type":"library","timestamp":"2026-04-16T12:00:00Z","data":{...}}
```

Parsing com `jaq`:

```bash
# Extrair todos os IDs de bibliotecas
context7 library react --json | jaq -r 'select(.type == "library") | .data.id'

# Filtrar por tipo
context7 docs /reactjs/react.dev --query "hooks" --json | jaq 'select(.type == "snippet")'
```

---

### Tratamento de sinais (v0.5.0+)

- Ctrl+C envia SIGINT no Unix e CTRL_C_EVENT no Windows
- A `context7-cli` captura o sinal, executa cleanup e encerra com codigo 130
- Requisicoes HTTP em andamento sao canceladas de forma graceful
- Nenhuma saida parcial e escrita no stdout apos a interrupcao

---

### Fallback ASCII (v0.5.0+)

Quando stdout nao e TTY (redirecionado ou em pipe) ou `NO_COLOR` esta definida, a `context7-cli` substitui decoracoes Unicode por equivalentes ASCII:

| Unicode | ASCII | Contexto |
|---------|-------|----------|
| `\u25b8` | `>` | Marcadores |
| `\u2500` | `-` | Linhas horizontais |

Isso garante saida limpa em terminais sem suporte a Unicode e em logs de CI/CD.

---

### Targets ARM Linux (v0.5.0+)

Binarios pre-compilados estao disponiveis para ARM64 Linux:

| Target | Caso de uso |
|--------|-------------|
| `aarch64-unknown-linux-gnu` | Raspberry Pi OS, Ubuntu em ARM, AWS Graviton |
| `aarch64-unknown-linux-musl` | Alpine Linux em ARM, containers minimais |

```bash
# Baixar binario ARM64 glibc
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-aarch64-unknown-linux-gnu.tar.gz \
  | tar xz
chmod +x context7 && mv context7 /usr/local/bin/

# Baixar binario ARM64 musl estatico (Alpine, containers)
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-aarch64-unknown-linux-musl.tar.gz \
  | tar xz
chmod +x context7 && mv context7 /usr/local/bin/
```

---

### Passo 10 — Autocompletar no Shell

Habilite o autocompletar por tab para o `context7` no seu shell. Instale uma vez — funciona para sempre.

#### Bash

```bash
# Instalar para o usuário atual
context7 completions bash > ~/.local/share/bash-completion/completions/context7

# Recarregar (ou abrir novo terminal)
source ~/.bashrc
```

#### Zsh

```bash
# Adicione ~/.zfunc ao fpath (coloque no ~/.zshrc se ainda não estiver):
# fpath=(~/.zfunc $fpath)
# autoload -Uz compinit && compinit

mkdir -p ~/.zfunc
context7 completions zsh > ~/.zfunc/_context7

# Recarregar completions
compinit
```

#### Fish

```bash
context7 completions fish > ~/.config/fish/completions/context7.fish
# Os completions são carregados automaticamente no próximo início do shell
```

#### PowerShell

```powershell
# Adicionar ao $PROFILE (executado a cada início do shell)
context7 completions powershell >> $PROFILE

# Recarregar o profile
. $PROFILE
```

#### Elvish

```bash
context7 completions elvish > ~/.config/elvish/lib/context7.elv
# Importe no ~/.config/elvish/rc.elv:
# use context7
```

---

### System prompt para LLMs (Português)

Copie e cole isto no system prompt do seu LLM para habilitar busca automática de documentação:

```
Você tem acesso à CLI `context7` para buscar documentação técnica atualizada de bibliotecas e frameworks.

## Ferramenta disponível: `context7`

### Quando usar
- Quando precisar de documentação de qualquer biblioteca (React, Vue, Axum, Tokio, etc.)
- Quando o usuário perguntar sobre APIs, configurações ou uso de uma biblioteca específica
- Quando sua base de conhecimento puder estar desatualizada sobre uma biblioteca
- Antes de sugerir código que dependa de APIs externas

### Como usar: três subcomandos

#### Subcomando `library` — Descobrir o ID de uma biblioteca
```bash
context7 library <NOME> [CONTEXTO_OPCIONAL]
```
- `NOME`: nome da biblioteca (ex: `react`, `axum`, `tokio`, `vue`)
- `CONTEXTO_OPCIONAL`: texto para refinar o ranking (ex: `"hooks de efeito"`)
- Retorna: lista de bibliotecas com IDs, títulos e pontuação de confiança
- **Sempre use este subcomando primeiro para obter o ID correto**

Exemplos:
```bash
context7 library react "hooks e estado"
context7 library axum "middleware e rotas"
context7 library tokio "canal assíncrono"
```

#### Subcomando `docs` — Buscar documentação por ID
```bash
context7 docs <LIBRARY_ID> [--query <TEXTO>] [--text]
```
- `LIBRARY_ID`: ID no formato `/org/repo` obtido via `library` (ex: `/reactjs/react.dev`)
- `--query <TEXTO>`: tópico específico a buscar (opcional mas recomendado)
- `--text`: retorna texto plano (recomendado para inserir no contexto)
- `--json`: retorna JSON estruturado (incompatível com `--text`)

Exemplos:
```bash
context7 docs /reactjs/react.dev --query "useEffect e cleanup" --text
context7 docs /tokio-rs/tokio --query "spawn_blocking" --text
context7 docs /rust-lang/rust --query "lifetimes e borrowing" --text
```

### Fluxo correto em 2 passos

**Passo 1:** Descubra o ID
```bash
context7 library react "hooks de efeito"
# → Retorna: /reactjs/react.dev, /preactjs/preact, ...
```

**Passo 2:** Busque documentação com o ID correto
```bash
context7 docs /reactjs/react.dev --query "useEffect dependências" --text
```

### Regras importantes
- **NUNCA invente um LIBRARY_ID** — sempre use `library` primeiro para obtê-lo
- `--text` e `--json` são incompatíveis: use apenas um
- A flag `--json` é global e funciona em qualquer subcomando
- Se `library` não retornar resultados, tente variações do nome (ex: `react` vs `reactjs`)
- Erros 401/403 indicam problema com a chave de API (verifique via `context7 keys list`)
- Erros 429 indicam rate limit: a CLI faz retry automaticamente

### Pré-requisito
- Chave de API configurada via `context7 keys add <CHAVE>` (recomendado)
- Binário `context7` disponível no PATH
```
