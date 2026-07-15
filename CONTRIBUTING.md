# Contributing to context7-cli / Contribuindo para o context7-cli

---

## English

Thank you for your interest in contributing. This document covers everything you need to get started.

### Prerequisites

- **Rust 1.75+** — install via [rustup.rs](https://rustup.rs)
- **cargo-deny** — `cargo install cargo-deny` (used in CI for license and advisory checks)
- **cargo-llvm-cov** *(optional, for coverage)* — `cargo install cargo-llvm-cov`

Verify your setup:

```bash
rustc --version   # should be >= 1.75
cargo --version
cargo deny --version
```

### Getting started

```bash
# Clone the repository
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli

# Verify it compiles
cargo check

# Run all tests
cargo test
```

### Project structure

| Module | Path | Responsibility |
|--------|------|----------------|
| `api` | `src/api.rs` | HTTP client — Context7 REST API calls, retry logic |
| `cli` | `src/cli.rs` | clap definitions — subcommands, flags, argument types |
| `errors` | `src/errors.rs` | `thiserror` error types — `ErroContext7` enum |
| `i18n` | `src/i18n.rs` | Bilingual strings — `Mensagem` enum, `t()` function |
| `output` | `src/output.rs` | All terminal I/O — the **only** module allowed to call `println!` |
| `storage` | `src/storage.rs` | Key management — TOML config, XDG paths, env vars |

> **Rule**: `output.rs` is the single source of truth for all terminal output. No other module may call `println!`, `print!`, or `eprintln!` directly — use `tracing::` macros for diagnostics and add a new function to `output.rs` for user-facing messages.

### Running tests

```bash
# Run all tests (unit + integration)
cargo test

# Run only integration tests
cargo test --test cli_integration

# Run with log output visible
RUST_LOG=debug cargo test -- --nocapture

# Run with coverage report
cargo llvm-cov --text
```

The test suite has 219+ tests. CI requires zero failures and ≥80% line coverage on business-logic files (`storage.rs`, `errors.rs`, `i18n.rs`, `output.rs`).

### Code style

- **Language**: field names in structs, enum variants, variable names, and log messages **must be in Brazilian Portuguese** — e.g., `chave_api`, `Mensagem::NenhumaChaveConfigurada`, `"Chave adicionada com sucesso."`.
- **Error handling**: use `anyhow::Result` in binaries and `thiserror` for structured error types in `errors.rs`. Never use `.unwrap()` or `.expect()` in production code — propagate with `?`.
- **Output**: all user-facing messages go through `output.rs`. All log messages go through `tracing::` macros.
- **Formatting**: `cargo fmt` is enforced in CI. Run it before committing.
- **Lints**: `cargo clippy -- -D warnings` must pass with zero warnings.

### Commit conventions

- Messages must be **bilingual** (English summary + Portuguese body, or a single clear English line).
- Follow [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format when updating `CHANGELOG.md`.
- **No `Co-authored-by:` lines from bots** (Renovate, Claude, Anthropic, `[bot]`, or any `*bot*` account). The repository has a `commit-msg` hook that rejects such commits.
- Commit message example:

```
context7-cli v0.4.0: shell completions + dynamic user-agent

- Adds `context7 completions <SHELL>` for bash/zsh/fish/powershell/elvish
- User-agent now reads version from env!("CARGO_PKG_VERSION") at compile time
- 64/64 HOW_TO_USE empirical tests passed
```

### PR checklist

Before opening a pull request, verify:

- [ ] `cargo check` — zero errors
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — zero formatting differences
- [ ] `cargo doc --no-deps` — zero documentation warnings
- [ ] `cargo test` — zero failing tests
- [ ] `CHANGELOG.md` updated with your changes under `[Unreleased]` or the appropriate version
- [ ] New user-facing strings added to `i18n.rs` with both EN and PT translations
- [ ] No `println!`/`print!` outside `output.rs`
- [ ] No `.unwrap()` or `.expect()` in production code paths

### License

By contributing, you agree that your contributions will be dual-licensed under **MIT OR Apache-2.0**, the same license as this project.

---

## Português

Obrigado pelo interesse em contribuir. Este documento cobre tudo o que você precisa para começar.

### Pré-requisitos

- **Rust 1.75+** — instale via [rustup.rs](https://rustup.rs)
- **cargo-deny** — `cargo install cargo-deny` (usado no CI para verificações de licença e advisories)
- **cargo-llvm-cov** *(opcional, para cobertura)* — `cargo install cargo-llvm-cov`

Verifique seu ambiente:

```bash
rustc --version   # deve ser >= 1.75
cargo --version
cargo deny --version
```

### Primeiros passos

```bash
# Clonar o repositório
git clone https://github.com/danilo-aguiar-br/context7-cli
cd context7-cli

# Verificar que compila
cargo check

# Executar todos os testes
cargo test
```

### Estrutura do projeto

| Módulo | Caminho | Responsabilidade |
|--------|---------|------------------|
| `api` | `src/api.rs` | Cliente HTTP — chamadas à API REST do Context7, lógica de retry |
| `cli` | `src/cli.rs` | Definições clap — subcomandos, flags, tipos de argumento |
| `errors` | `src/errors.rs` | Tipos de erro `thiserror` — enum `ErroContext7` |
| `i18n` | `src/i18n.rs` | Strings bilíngues — enum `Mensagem`, função `t()` |
| `output` | `src/output.rs` | Todo I/O de terminal — o **único** módulo autorizado a chamar `println!` |
| `storage` | `src/storage.rs` | Gerenciamento de chaves — config TOML, caminhos XDG, variáveis de ambiente |

> **Regra**: `output.rs` é a única fonte de verdade para todo output de terminal. Nenhum outro módulo pode chamar `println!`, `print!` ou `eprintln!` diretamente — use macros `tracing::` para diagnósticos e adicione uma nova função em `output.rs` para mensagens voltadas ao usuário.

### Executando os testes

```bash
# Executar todos os testes (unitários + integração)
cargo test

# Executar apenas testes de integração
cargo test --test cli_integration

# Executar com output de log visível
RUST_LOG=debug cargo test -- --nocapture

# Executar com relatório de cobertura
cargo llvm-cov --text
```

A suite de testes tem 219+ testes. O CI exige zero falhas e ≥80% de cobertura de linhas nos arquivos de lógica de negócio (`storage.rs`, `errors.rs`, `i18n.rs`, `output.rs`).

### Estilo de código

- **Idioma**: nomes de campos em structs, variantes de enum, nomes de variáveis e mensagens de log **devem estar em português brasileiro** — ex: `chave_api`, `Mensagem::NenhumaChaveConfigurada`, `"Chave adicionada com sucesso."`.
- **Tratamento de erros**: use `anyhow::Result` em binários e `thiserror` para tipos de erro estruturados em `errors.rs`. Nunca use `.unwrap()` ou `.expect()` em código de produção — propague com `?`.
- **Output**: todas as mensagens voltadas ao usuário passam por `output.rs`. Todas as mensagens de log passam pelas macros `tracing::`.
- **Formatação**: `cargo fmt` é obrigatório no CI. Execute antes de fazer commit.
- **Lints**: `cargo clippy -- -D warnings` deve passar com zero warnings.

### Convenções de commit

- Mensagens devem ser **bilíngues** (resumo em inglês + corpo em português, ou uma linha clara em inglês).
- Siga o formato [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) ao atualizar o `CHANGELOG.md`.
- **Sem linhas `Co-authored-by:` de bots** (Renovate, Claude, Anthropic, `[bot]`, ou qualquer conta `*bot*`). O repositório tem um hook `commit-msg` que rejeita esses commits.
- Exemplo de mensagem de commit:

```
context7-cli v0.4.0: shell completions + user-agent dinâmico

- Adiciona `context7 completions <SHELL>` para bash/zsh/fish/powershell/elvish
- User-agent agora lê a versão via env!("CARGO_PKG_VERSION") em tempo de compilação
- 64/64 testes empíricos do HOW_TO_USE aprovados
```

### Checklist para PRs

Antes de abrir um pull request, verifique:

- [ ] `cargo check` — zero erros
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo fmt --check` — zero diferenças de formatação
- [ ] `cargo doc --no-deps` — zero warnings de documentação
- [ ] `cargo test` — zero testes falhando
- [ ] `CHANGELOG.md` atualizado com suas mudanças em `[Unreleased]` ou na versão adequada
- [ ] Novas strings voltadas ao usuário adicionadas ao `i18n.rs` com traduções EN e PT
- [ ] Sem `println!`/`print!` fora de `output.rs`
- [ ] Sem `.unwrap()` ou `.expect()` em caminhos de código de produção

### Licença

Ao contribuir, você concorda que suas contribuições serão licenciadas sob a licença dual **MIT OR Apache-2.0**, a mesma licença deste projeto.
