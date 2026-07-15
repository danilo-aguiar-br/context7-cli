# crates.md — Manual Imperativo para Replicação de Projetos Rust CLI


> Destilação do conhecimento acumulado no `context7-cli v0.5.0`.
> Linguagem: regras INVIOLÁVEIS — DEVE, JAMAIS, OBRIGATÓRIO, PROIBIDO, SEMPRE, NUNCA.
> Para quem: desenvolvedor (humano ou agente) que replica um novo projeto Rust CLI cross-platform.

---

## Sumário

1. [Preâmbulo — Missão e Escopo deste Manual](#1-preâmbulo--missão-e-escopo-deste-manual)
2. [Estrutura de Arquivos Obrigatória](#2-estrutura-de-arquivos-obrigatória)
3. [MSRV — Pinning Inviolável](#3-msrv--pinning-inviolável)
4. [Sistema de Gestão de Variáveis CLI Cross-Platform](#4-sistema-de-gestão-de-variáveis-cli-cross-platform)
5. [Como Replicar o Padrão para QUALQUER Variável CLI](#5-como-replicar-o-padrão-para-qualquer-variável-cli)
6. [i18n Bilíngue + Detecção de Locale Cross-Platform](#6-i18n-bilíngue--detecção-de-locale-cross-platform)
7. [Testes — Pirâmide Obrigatória](#7-testes--pirâmide-obrigatória)
8. [Cargo.toml — Anatomia Obrigatória](#8-cargotoml--anatomia-obrigatória)
9. [Integração com docs.rs](#9-integração-com-docsrs)
10. [`.gitignore` — Blindagem do Repositório](#10-gitignore--blindagem-do-repositório)
11. [`Cargo.toml [exclude]` — Blindagem do Pacote Publicado](#11-cargotoml-exclude--blindagem-do-pacote-publicado)
12. [Pastas Ignoradas — Convenção de Privacidade](#12-pastas-ignoradas--convenção-de-privacidade)
13. [Anti-Vazamento de Secrets — 3 Camadas](#13-anti-vazamento-de-secrets--3-camadas)
14. [Fluxo de Publicação GitHub Actions + crates.io](#14-fluxo-de-publicação-github-actions--cratesio)
15. [Copywriting do README — Estratégia AIDA Bilíngue](#15-copywriting-do-readme--estratégia-aida-bilíngue)
16. [Bloqueio de Co-authored-by — Claude + GitHub](#16-bloqueio-de-co-authored-by--claude--github)
17. [Checklist Final de Replicação](#17-checklist-final-de-replicação)

---

## 1. Preâmbulo — Missão e Escopo deste Manual

**Princípio absoluto**: este documento é um manual de regras, não um tutorial. Cada afirmação é uma regra OBRIGATÓRIA ou uma PROIBIÇÃO. Zero narrativa decorativa. Zero jargão sem definição.

### Para quem é

DEVE ser lido por qualquer desenvolvedor (humano ou agente de IA) que vá criar ou manter um projeto Rust CLI que:

- Publica em **crates.io** com suporte a docs.rs
- Roda em **Linux, macOS e Windows** sem dependências de runtime
- Gerencia **configuração persistente cross-platform** (chaves de API, tokens, endpoints)
- Exige **internacionalização bilíngue** (EN + PT-BR)
- Mantém **isolamento completo entre testes** via `tempfile`
- Aplica **3 camadas de anti-vazamento** de secrets

### Quando aplicar

DEVE aplicar este manual:

- Ao iniciar qualquer projeto Rust CLI publicado em crates.io
- Ao replicar o padrão `context7-cli` para outro projeto (ex.: `my-tool-cli`)
- Ao revisar conformidade de projeto existente com estas regras
- Ao delegar implementação para agente de IA

### O que este manual NÃO é

JAMAIS use este manual como:

- Substituto para `cargo check` + `cargo clippy` — eles são OBRIGATÓRIOS separadamente
- Documentação da API pública — use docs.rs para isso
- Tutorial de Rust para iniciantes — pré-requisito: conhecimento básico de ownership e traits

---

## 2. Estrutura de Arquivos Obrigatória

**Princípio absoluto**: cada arquivo tem UMA responsabilidade exclusiva. JAMAIS misture responsabilidades. O módulo `output.rs` é o ÚNICO ponto de I/O terminal.

### OBRIGATÓRIO — Árvore de src/

```
src/
├── lib.rs          ← Fachada pública: pub mod, run(), inicializar_logging()
├── main.rs         ← Ponto de entrada: chama run() + mantém GuardaLog vivo
├── cli.rs          ← Structs Clap: Parser, Subcommand, args — ZERO lógica de negócio
├── api.rs          ← Cliente HTTP: reqwest, retry, desserialização de resposta
├── errors.rs       ← Enum ErroContext7 com thiserror — TODOS os erros do domínio
├── i18n.rs         ← Enum Mensagem bilíngue — ÚNICA fonte de strings de UI
├── storage.rs      ← Persistência XDG: caminho, leitura, escrita, operações CRUD
├── output.rs       ← Terminal output — ÚNICO módulo autorizado a usar println!
└── platform.rs     ← Inicialização de console cross-platform (UTF-8, ANSI, code pages)
```

Ver `src/lib.rs:1-16` para tabela de responsabilidades com links de documentação.

### OBRIGATÓRIO — Árvore de tests/

```
tests/
├── cli_integration.rs       ← Testes E2E do binário via assert_cmd
├── api_integration.rs       ← Testes de API com wiremock (ZERO chamadas reais)
├── i18n_integration.rs      ← Testes das variantes de Mensagem bilíngues
└── storage_integration.rs   ← Testes do CRUD XDG com TempDir isolado
```

### OBRIGATÓRIO — Responsabilidade de cada módulo

| Módulo       | Pode fazer                              | JAMAIS pode fazer            |
| ------------ | --------------------------------------- | ---------------------------- |
| `lib.rs`     | Reexportar módulos, entry point `run()` | Lógica de negócio, I/O       |
| `main.rs`    | Chamar `run()`, manter `GuardaLog`      | Qualquer lógica              |
| `cli.rs`     | Definir structs Clap                    | Chamar API, ler config       |
| `api.rs`     | HTTP, retry, parse de resposta          | Imprimir, gerenciar config   |
| `errors.rs`  | Definir tipos de erro                   | Imprimir, chamar API         |
| `i18n.rs`    | Retornar strings localizadas            | Imprimir, chamar I/O         |
| `storage.rs` | Ler/escrever config XDG                 | Imprimir diretamente         |
| `output.rs`  | Imprimir para stdout/stderr             | Chamar API, gerenciar config |
| `platform.rs`| Inicializar console (UTF-8, ANSI, code pages) | Lógica de negócio, I/O terminal |

### OBRIGATÓRIO — Regra do output.rs único

DEVE centralizar TODO output de terminal em `output.rs`. JAMAIS escreva `println!` em `api.rs`, `storage.rs`, `cli.rs` ou `errors.rs`. O módulo `output.rs` é o ÚNICO ponto de contato com stdout/stderr.

**Verificação obrigatória**:

```bash
sg -p 'println!($$$ARGS)' -l rust src/api.rs src/storage.rs src/cli.rs src/errors.rs src/platform.rs
```

DEVE retornar zero resultados. Se encontrar `println!` fora de `output.rs`, CORRIJA antes de prosseguir.

### OBRIGATÓRIO — Hierarquia lib.rs → módulos → main.rs

```rust
// lib.rs — SEMPRE começa com doc comments //! estruturados
//! context7-cli library crate.
//!
//! # Module overview
//! | Module | Responsibility |
//! |---|---|
//! | [`errors`] | Structured error types |
//! | [`i18n`] | Bilingual i18n |
//! | [`storage`] | XDG config storage |
//! ...

pub mod api;
pub mod cli;
pub mod errors;
pub mod i18n;
pub mod output;
pub mod platform;
pub mod storage;

pub fn run() -> anyhow::Result<()> { ... }
```

```rust
// main.rs — APENAS entry point
fn main() -> anyhow::Result<()> {
    let _guard = context7_cli::inicializar_logging()?;
    context7_cli::run()
}
```

JAMAIS coloque lógica de negócio em `main.rs`. O `_guard` de log DEVE ser mantido vivo até o fim de `main()`.

---

## 3. MSRV — Pinning Inviolável

**Princípio absoluto**: `rust-version` em `Cargo.toml` é INVIOLÁVEL. JAMAIS eleve o MSRV sem decisão consciente documentada no CHANGELOG. JAMAIS use crate que exija Rust superior ao MSRV declarado.

### OBRIGATÓRIO — Declaração em Cargo.toml

```toml
[package]
rust-version = "1.75"
```

DEVE validar no CI com job dedicado `msrv` (ver `ci.yml:msrv` job). O job DEVE executar:

```bash
rg -q '^rust-version\s*=\s*"1\.75"' Cargo.toml || exit 1
```

Ver `.github/workflows/ci.yml` job `msrv` para implementação completa com mensagem de erro descritiva.

### OBRIGATÓRIO — Pinning exato de crates problemáticas

Algumas crates elevam seu próprio MSRV sem SemVer major. DEVE usar pinning `=x.y.z` para:

| Crate           | Pin       | Motivo                                                                   |
| --------------- | --------- | ------------------------------------------------------------------------ |
| `clap`          | `=4.5.32` | `clap 4.5.33+` puxa `clap_lex 1.x` (edition2024, requer Rust 1.85)       |
| `clap_complete` | `=4.5.46` | Alinhado à série clap 4.5.x                                              |
| `toml`          | `=0.8.23` | `toml 0.9.x+` eleva `rust_version` para 1.76; `toml 1.x` usa edition2024 |
| `assert_cmd`    | `=2.1.2`  | `assert_cmd 2.1.3+` migrou para edition2024 (Rust 1.85)                  |

Ver `Cargo.toml:31,34,44,61` para os pins em uso.

### OBRIGATÓRIO — Sintaxe de pin no Cargo.toml

```toml
[dependencies]
# Sempre comentar o MOTIVO do pin
clap = { version = "=4.5.32", features = ["derive", "env", "color"] }
toml = "=0.8.23"  # toml 0.9.x+ eleva rust_version para 1.76

[dev-dependencies]
assert_cmd = "=2.1.2"  # 2.1.3+ migrou para edition2024 (Rust 1.85)
```

### PROIBIDO — MSRV

- JAMAIS eleve o MSRV sem atualizar `rust-version` no `Cargo.toml`
- JAMAIS adicione crate `x.y.z` que exija Rust > MSRV sem pin ou justificativa
- JAMAIS use `cargo update` indiscriminadamente — verifique MSRV das versões novas
- JAMAIS omita comentário explicativo em todo pin `=x.y.z`

### Verificação antes de adicionar qualquer nova dependência

```bash
# Verificar rust-version da crate antes de adicionar
cargo add nome-da-crate --dry-run
rg 'rust-version\|rust_version\|edition' ~/.cargo/registry/src/**/nome-da-crate-*/Cargo.toml
```

---

## 4. Sistema de Gestão de Variáveis CLI Cross-Platform

**Princípio absoluto**: NUNCA hardcode paths de configuração. SEMPRE use `directories::ProjectDirs` para caminhos XDG. SEMPRE implemente as 4 camadas de precedência na ORDEM EXATA abaixo.

### 4.1 — Arquitetura de 4 Camadas de Precedência

DEVE implementar exatamente nesta ordem (maior → menor prioridade):

```
1. Variável de ambiente RUNTIME   →  CRATE_API_KEYS (ou CRATE_CONFIG)
2. Config XDG                     →  ~/.config/crate/config.toml
3. .env no CWD                    →  CONTEXT7_API=value (formato dotenv)
4. Variável de ambiente COMPILE   →  option_env!("CRATE_API_KEYS")
```

Ver `src/storage.rs:220-265` para implementação completa da função `carregar_chaves_api()`.

```rust
pub fn carregar_chaves_api() -> Result<Vec<String>> {
    // Camada 1: env var runtime (maior prioridade)
    if let Some(chaves) = ler_env_var_chave() {
        return Ok(chaves);
    }
    // Camada 2: config XDG
    match ler_config_xdg() {
        Ok(Some(chaves)) => return Ok(chaves),
        Ok(None) => {}
        Err(e) => warn!("Falha ao ler XDG (continuando): {}", e),
    }
    // Camada 3: .env no CWD
    if let Some(chaves) = ler_env_cwd() {
        return Ok(chaves);
    }
    // Camada 4: compile-time (menor prioridade)
    if let Some(chaves) = ler_env_compile_time() {
        return Ok(chaves);
    }
    bail!(t(Mensagem::NenhumaChaveConfigurada))
}
```

### 4.2 — Crate `directories` OBRIGATÓRIA

DEVE usar `directories::ProjectDirs` para TODOS os caminhos de configuração. JAMAIS hardcode `~/.config/`, `%APPDATA%` ou `~/Library/`.

```rust
use directories::ProjectDirs;

pub fn descobrir_caminho_config() -> Option<PathBuf> {
    // Verificar override testável primeiro
    if let Some(base) = resolver_home_override() {
        return Some(base.join("context7").join("config.toml"));
    }
    // Fallback XDG via directories
    ProjectDirs::from("", "", "context7")
        .map(|dirs| dirs.config_dir().join("config.toml"))
}
```

Caminhos resultantes por plataforma:

| Plataforma | Caminho                                                                     |
| ---------- | --------------------------------------------------------------------------- |
| Linux      | `$XDG_CONFIG_HOME/context7/config.toml` ou `~/.config/context7/config.toml` |
| macOS      | `~/Library/Application Support/context7/config.toml`                        |
| Windows    | `%APPDATA%\context7\config.toml`                                            |

Ver `src/storage.rs:49,101` para implementação de `descobrir_caminho_config()` e `descobrir_caminho_logs_xdg()`.

### 4.3 — Override Testável via Variável de Ambiente

DEVE implementar `<CRATE>_HOME` como override do diretório base. Isso PERMITE que testes usem `TempDir` sem contaminar configuração real. DEVE rejeitar valores com componentes `..` para evitar path traversal — `tracing::warn!` ao detectar. Ver `src/storage.rs:71-89` para implementação completa com validação anti-path-traversal.

### 4.4 — Permissões Unix 0o600

DEVE aplicar `chmod 600` em TODA escrita de config no Unix via `PermissionsExt::set_mode(0o600)` dentro de bloco `#[cfg(unix)]`. JAMAIS grave arquivo de configuração sensível sem esta proteção. DEVE chamar `aplicar_permissoes_600` APÓS cada `std::fs::write` de config. Ver `src/storage.rs:48-62` para implementação centralizada (chamada por `escrever_config_xdg` e `escrever_config_arquivo`).

### 4.5 — Persistência TOML com schema_version

DEVE usar TOML para o arquivo de configuração. DEVE incluir `schema_version: u32` em `ConfigArquivo` e `added_at: String` (RFC 3339) em `ChaveArmazenada`. Ver `src/storage.rs:19-40` para definição completa.

DEVE usar nomes em inglês (`value`, `added_at`, `schema_version`, `keys`) para campos que aparecem no TOML externo. O formato externo DEVE ser estável e previsível para usuários que editam manualmente. JAMAIS renomeie campos do TOML externo entre versões sem incrementar `schema_version`.

### 4.6 — Timestamp RFC 3339 via chrono

DEVE usar `chrono::Utc::now().to_rfc3339()` para todos os timestamps. JAMAIS use formatos locais, Unix epoch bruto sem formatação, ou strings de data ambíguas. Ver `src/storage.rs:311` para uso real em `escrever_config_xdg`.

### 4.7 — Mascaramento Seguro de Valores Sensíveis

JAMAIS exiba chaves/tokens completos em stdout, stderr, logs ou output de `list`. DEVE mascarar mostrando primeiros 12 + últimos 4 caracteres. DEVE usar `chars()` — NUNCA indexação por bytes — para segurança UTF-8. Chaves com `≤16` chars retornam `"***"`. Ver `src/storage.rs:357-387` para implementação completa.

### 4.8 — Deduplicação Automática no add

DEVE verificar duplicata com `config.keys.iter().any(|c| c.value == chave_trimmed)` ANTES de escrever. DEVE exibir aviso específico via i18n (sem erro fatal). Ver `src/storage.rs:423-444` para `cmd_keys_add` completo.

### 4.9 — CRUD Completo: 8 operações OBRIGATÓRIAS

DEVE implementar exatamente estas 8 operações no subcomando `keys`:

| Operação           | Função                | Descrição                                   |
| ------------------ | --------------------- | ------------------------------------------- |
| `add <chave>`      | `cmd_keys_add`        | Adiciona com deduplicação e chmod 600       |
| `list [--json]`    | `cmd_keys_list`       | Lista com mascaramento; `--json` para pipes |
| `remove <N>`       | `cmd_keys_remove`     | Remove por índice 1-based                   |
| `clear [--yes]`    | `cmd_keys_clear`      | Remove todas; confirma sem `--yes`          |
| `path`             | `cmd_keys_path`       | Exibe caminho do arquivo de config          |
| `import <arquivo>` | `cmd_keys_import`     | Importa de `.env` (formato `KEY=valor`)     |
| `export`           | `cmd_keys_export`     | Exporta para stdout em formato `.env`       |
| (implícito)        | `carregar_chaves_api` | Carrega via hierarquia de 4 camadas         |

Ver `src/storage.rs:423-579` para todas as funções `cmd_keys_*`.

### 4.10 — Exemplo Completo de Uso

```bash
context7 keys add ctx7sk-...          # adiciona com deduplicação + chmod 600
context7 keys list                     # lista mascarado: ctx7sk-YOUR-KE...HERE
context7 keys list --json | jaq '.[0].masked_key'  # para pipes
context7 keys remove 2                 # remove por índice 1-based
context7 keys export > backup.env      # exporta para .env
context7 keys import backup.env        # importa de .env
context7 keys path                     # exibe caminho do config.toml
context7 keys clear --yes              # remove todas sem prompt
```

---

## 5. Como Replicar o Padrão para QUALQUER Variável CLI

**Princípio absoluto**: o padrão de gestão de chaves de API é um TEMPLATE GENÉRICO. Qualquer configuração persistente cross-platform (proxy, timeout, endpoint, token) DEVE seguir exatamente este template.

### Modelo Conceitual

"Variável de configuração persistente cross-platform com precedência multicamada" = qualquer dado que:

1. O usuário configura UMA VEZ e reutiliza em múltiplas invocações
2. Pode ser sobrescrito temporariamente via env var sem editar o arquivo
3. Precisa funcionar em Linux, macOS e Windows sem paths hardcoded
4. NUNCA deve aparecer em logs completo (mascarar se sensível)

### Template de Struct Parametrizável

DEVE definir `MinhaConfigArmazenada { value: String, added_at: String }` e `MinhaConfigArquivo { schema_version: u32, #[serde(default)] items: Vec<MinhaConfigArmazenada> }`. Campos em inglês no TOML externo. Ver padrão completo em `src/storage.rs:19-40`.

### 7 Passos Obrigatórios de Replicação

| Passo | Ação                                                                                   | Fonte no projeto                    |
| ----- | -------------------------------------------------------------------------------------- | ----------------------------------- |
| 1     | Definir env var `<CRATE_UPPER>_HOME` com validação anti-path-traversal                 | `src/storage.rs:71-89`              |
| 2     | Implementar `descobrir_caminho_config()` via `ProjectDirs::from("", "", "crate-name")` | `src/storage.rs:101`                |
| 3     | Implementar `aplicar_permissoes_600()` com `#[cfg(unix)]`                              | `src/storage.rs:48-62`              |
| 4     | Implementar hierarquia de 4 camadas em `carregar_minha_config()`                       | `src/storage.rs:220-265`            |
| 5     | Implementar CRUD: add + list + remove + clear + path + import + export                 | `src/storage.rs:423-579`            |
| 6     | Implementar mascaramento de valores sensíveis                                          | `src/storage.rs:357-387`            |
| 7     | Escrever testes com `TempDir` + `env_clear()` + `#[serial]`                            | `tests/storage_integration.rs:1-50` |

JAMAIS pule qualquer passo. JAMAIS reordene. Cada passo depende do anterior.

### Exemplos de Replicação

| Caso de uso          | Env var runtime    | Arquivo XDG   | Override testável |
| -------------------- | ------------------ | ------------- | ----------------- |
| Chaves de API        | `CRATE_API_KEYS`   | `config.toml` | `CRATE_HOME`      |
| Token OAuth          | `CRATE_TOKEN`      | `auth.toml`   | `CRATE_HOME`      |
| URL de proxy         | `CRATE_PROXY`      | `config.toml` | `CRATE_HOME`      |
| Timeout (ms)         | `CRATE_TIMEOUT_MS` | `config.toml` | `CRATE_HOME`      |
| Endpoint customizado | `CRATE_ENDPOINT`   | `config.toml` | `CRATE_HOME`      |

---

## 6. i18n Bilíngue + Detecção de Locale Cross-Platform

**Princípio absoluto**: JAMAIS escreva strings de UI fora de `i18n.rs`. Toda string visível ao usuário DEVE ter variante em EN e PT. O módulo `i18n.rs` é a ÚNICA fonte de verdade para texto de interface.

### 6.1 — Enum `Idioma` com variantes explícitas

DEVE usar enum com variantes nomeadas. JAMAIS use `bool` ou `String` para representar idioma.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Idioma {
    English,
    Portugues,
}
```

Ver `src/i18n.rs:17-22` para definição exata.

### 6.2 — Enum `Mensagem` como única fonte de strings

DEVE representar CADA mensagem de UI como uma variante do enum `Mensagem`. JAMAIS escreva strings literais de UI em `storage.rs`, `api.rs`, `cli.rs` ou `output.rs`.

```rust
pub enum Mensagem {
    ChaveAdicionada,
    ChaveJaExistia,
    NenhumaChaveArmazenada,
    // ... 52 variantes no total (ver src/i18n.rs:81-203)
}
```

### 6.3 — Funções en() e pt() com match exaustivo

DEVE implementar duas funções privadas `en(msg: Mensagem) -> &'static str` e `pt(msg: Mensagem) -> &'static str` com `match` exaustivo sobre todas as variantes. O compilador GARANTE que nenhuma variante fica sem tradução. Ver `src/i18n.rs:229-395` (en) e `src/i18n.rs:399-544` (pt).

### 6.4 — Precedência de Resolução (4 camadas)

DEVE implementar exatamente nesta ordem: (1) flag `--lang` CLI, (2) `CONTEXT7_LANG` env, (3) `sys_locale::get_locale()` com prefixo `"pt*"`, (4) default `English`. Ver `src/i18n.rs:46-66` para implementação completa.

### 6.5 — OnceLock para Thread-Safety sem Mutex

DEVE usar `static IDIOMA_GLOBAL: OnceLock<Idioma> = OnceLock::new()` para estado global de idioma. JAMAIS use `Mutex` para dado escrito UMA VEZ na inicialização e lido N vezes. `definir_idioma` ignora silenciosamente chamadas duplicadas — semântica correta. Ver `src/i18n.rs:25,35-37` para definição e funções de acesso.

### 6.6 — Crate sys-locale OBRIGATÓRIA

DEVE usar `sys-locale = "0.3"` para detectar locale do sistema de forma cross-platform. Esta crate usa:

- Linux/macOS: variáveis `LANG`, `LC_ALL`, `LC_MESSAGES`
- Windows: `GetUserDefaultLocaleName()` via Windows API

JAMAIS leia `LANG` ou `LC_ALL` diretamente — o comportamento difere entre plataformas. É OBRIGATÓRIO depender do `sys-locale` para portabilidade garantida.

### 6.7 — Testes Bilíngues de Todas as Variantes

DEVE testar que CADA variante de `Mensagem` retorna string não-vazia em ambos os idiomas via `variante.texto(Idioma::English).is_empty()` e `variante.texto(Idioma::Portugues).is_empty()`. DEVE verificar que traduções PT não são iguais às EN. Ver `tests/i18n_integration.rs` para testes completos.

### 6.8 — Função de Acesso t() para Código de Produção

DEVE usar `t(Mensagem::Variante)` em produção (usa estado global). DEVE usar `Mensagem::texto(idioma)` em testes (determinístico, sem estado global). Ex.: `bail!(t(Mensagem::NenhumaChaveConfigurada))` e `assert_eq!(Mensagem::ChaveAdicionada.texto(Idioma::English), "Key added successfully at: ")`.

### PROIBIDO — i18n

- JAMAIS escreva string de UI fora de `i18n.rs` (ex.: `println!("Error: key not found")`)
- JAMAIS use `if idioma == "pt"` — SEMPRE use `match idioma_atual()`
- JAMAIS omita variante em uma das funções `en()` ou `pt()` — o compilador NÃO vai deixar
- JAMAIS use `String` quando `&'static str` basta (todas as variantes retornam `&'static str`)

---

## 7. Testes — Pirâmide Obrigatória

**Princípio absoluto**: ZERO testes sem isolamento completo. ZERO chamadas HTTP reais em testes. ZERO manipulação de arquivos do sistema real. SEMPRE usar `TempDir` + `env_clear()`.

### 7.1 — Testes Unitários Inline

DEVE escrever testes unitários no próprio arquivo (`#[cfg(test)] mod testes`). DEVE testar funções puras sem I/O.

```rust
#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn mascarar_chave_curta_retorna_asteriscos() {
        assert_eq!(mascarar_chave("abc"), "***");
    }

    #[test]
    fn extrair_chaves_env_ignora_comentarios() {
        let conteudo = "# comentário\nCONTEXT7_API=ctx7sk-abc\n";
        let chaves = extrair_chaves_env(conteudo).unwrap();
        assert_eq!(chaves.len(), 1);
    }
}
```

### 7.2 — Testes de Integração E2E

DEVE usar `assert_cmd::Command::cargo_bin("nome-do-binario")` para testar o binário compilado. DEVE usar `predicates` para assertions composáveis.

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn keys_add_exibe_confirmacao() {
    let dir = TempDir::new().unwrap();
    Command::cargo_bin("context7").unwrap()
        .env_clear()
        .env("CONTEXT7_HOME", dir.path())
        .args(["keys", "add", "ctx7sk-test-12345678901"])
        .assert()
        .success()
        .stdout(predicate::str::contains("adicionada"));
}
```

### 7.3 — Isolamento Obrigatório via TempDir

DEVE SEMPRE usar `tempfile::TempDir` como `CONTEXT7_HOME`. DEVE SEMPRE chamar `env_clear()` antes de definir env vars de teste. JAMAIS polua a configuração real do usuário.

```rust
fn cmd_xdg(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("context7").unwrap();
    cmd.env_clear()
       .env("CONTEXT7_HOME", dir.path())
       .env("HOME", dir.path());
    cmd
}
```

Ver `tests/storage_integration.rs:22-30` para helper `cmd_xdg()` — copiar EXATAMENTE.

**ATENÇÃO**: JAMAIS defina `CONTEXT7_API_KEYS=""` (string vazia) — os value_parsers do Clap rejeitam string vazia e o teste falha com erro confuso. O isolamento de chaves é garantido pelo `CONTEXT7_HOME` temporário vazio.

### 7.4 — Serialização via serial_test

DEVE marcar com `#[serial]` TODOS os testes que:

- Definem ou leem variáveis de ambiente (`env::set_var`, `env::var`)
- Escrevem ou leem do sistema de arquivos real (não `TempDir`)
- Chamam funções que dependem de estado global (`idioma_atual()`, `IDIOMA_GLOBAL`)

```rust
use serial_test::serial;

#[test]
#[serial]
fn testa_add_list_remove_ciclo_completo_via_xdg_home() {
    let dir = TempDir::new().unwrap();
    // ...
}
```

Ver `tests/storage_integration.rs:36-37` para uso real de `#[serial]`.

### 7.5 — HTTP Mocking via wiremock

JAMAIS faça chamadas HTTP reais em testes. DEVE usar `wiremock::MockServer` para interceptar toda comunicação de rede.

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn api_busca_biblioteca_retorna_resultados() {
    let mock_server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([...])))
        .mount(&mock_server)
        .await;
    // Apontar cliente para mock_server.uri()
}
```

### 7.6 — Cobertura Mínima 80%

DEVE executar `cargo llvm-cov` e atingir ≥80% de cobertura para código novo. JAMAIS aceite PR com cobertura abaixo de 80% sem justificativa explícita aprovada.

```bash
cargo llvm-cov --text
cargo llvm-cov --html --open  # para inspeção visual
```

Ver `.github/workflows/ci.yml` job `coverage` para configuração no CI.

### 7.7 — Execução em 3 OS via Matrix CI

DEVE executar testes completos em ubuntu-latest, macos-latest e windows-latest. JAMAIS aceite "funciona no meu Linux" sem evidência dos 3 sistemas.

```yaml
# ci.yml — job check
strategy:
  matrix:
    os: [ubuntu-latest, macos-latest, windows-latest]
```

Ver `.github/workflows/ci.yml` job `check` para configuração completa.

### 7.8 — Smoke Test musl para Alpine

DEVE incluir smoke test do binário musl (`x86_64-unknown-linux-musl`) para garantir compatibilidade com Alpine Linux e containers minimais.

```bash
# Build musl
cargo build --release --target x86_64-unknown-linux-musl

# Smoke test mínimo
./target/x86_64-unknown-linux-musl/release/context7 --version
./target/x86_64-unknown-linux-musl/release/context7 --help
```

Ver `.github/workflows/release.yml` job `build-matrix` para target musl na matrix.

### PROIBIDO — Testes

- JAMAIS `thread::sleep` em testes — use sincronização adequada (channels, barriers)
- JAMAIS chamadas HTTP reais — use `wiremock`
- JAMAIS `env::set_var` sem `#[serial]` — causa flaky tests
- JAMAIS `unwrap()` em helpers de teste sem comentário explicando por que é seguro
- JAMAIS ignore testes falhando — CORRIJA antes de prosseguir

---

## 8. Cargo.toml — Anatomia Obrigatória

**Princípio absoluto**: todo campo relevante DEVE estar declarado em `Cargo.toml`. Campos ausentes tornam o pacote menos descobrível no crates.io e degradam a experiência no docs.rs.

### OBRIGATÓRIO — Campos da seção [package]

```toml
[package]
name = "minha-crate-cli"          # OBRIGATÓRIO — nome exato no crates.io
version = "0.1.0"                  # OBRIGATÓRIO — SemVer estrito
edition = "2021"                   # OBRIGATÓRIO — sempre edition 2021
rust-version = "1.75"              # OBRIGATÓRIO — MSRV explícito (ver Seção 3)
default-run = "nome-do-binario"    # OBRIGATÓRIO se há [[bin]]

# Metadados obrigatórios para crates.io
description = "Uma linha clara do value proposition"
authors = ["Nome Completo <email@dominio.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/usuario/repo"
homepage = "https://github.com/usuario/repo"
documentation = "https://docs.rs/minha-crate-cli"
readme = "README.md"
keywords = ["keyword1", "keyword2", "keyword3"]  # máx 5, escolher bem
categories = ["command-line-utilities"]          # categoria crates.io
```

Ver `Cargo.toml:1-16` para todos os campos em uso no `context7-cli`.

### OBRIGATÓRIO — [lib] e [[bin]] explícitos

DEVE declarar explicitamente mesmo que os paths sejam os padrão (`src/lib.rs` e `src/main.rs`). Clareza é OBRIGATÓRIA.

```toml
[lib]
name = "minha_crate_cli"   # underscore — nome do módulo Rust
path = "src/lib.rs"

[[bin]]
name = "minha-crate"       # hífen — nome do binário CLI
path = "src/main.rs"
```

### OBRIGATÓRIO — [dependencies] com documentação

DEVE documentar o MOTIVO de cada pin `=x.y.z`. DEVE documentar por que feature flag está ativada.

```toml
[dependencies]
anyhow = "1"
clap = { version = "=4.5.32", features = ["derive", "env", "color"] }
# Pin: clap 4.5.33+ puxa clap_lex 1.x (edition2024, requer Rust 1.85)
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "http2"] }
# default-features = false: exclui openssl nativo — usa rustls para static link
fastrand = "2"  # Fisher-Yates shuffle para rotação de chaves — substitui rand 0.8
zeroize = { version = "1", features = ["derive"] }  # Limpeza segura de chaves API da memória ao drop
unicode-normalization = "0.1"  # Normalização NFC de paths para compatibilidade macOS HFS+
```

DEVE REMOVER `rand` da lista de dependências — substituído por `fastrand` que é mais leve e sem dependências transitivas

### OBRIGATÓRIO — Dependências condicionais por target

DEVE declarar allocator otimizado para targets musl:

```toml
[target.'cfg(target_env = "musl")'.dependencies]
mimalloc = { version = "0.1", default-features = false }
# Allocator otimizado para targets musl — 7x mais rápido que malloc do musl
```

### OBRIGATÓRIO — [profile.release] para builds otimizados

DEVE declarar profile de release com otimizações máximas:

```toml
[profile.release]
lto = "fat"           # Link-Time Optimization completa — binário menor e mais rápido
codegen-units = 1     # Compilação em unidade única — permite otimizações globais
panic = "abort"       # Sem unwind — binário menor, sem overhead de backtrace
strip = "symbols"     # Remove símbolos de debug — reduz tamanho final
```

### OBRIGATÓRIO — [dev-dependencies] isoladas

DEVE manter dependências de teste em `[dev-dependencies]`. JAMAIS adicione crate de teste em `[dependencies]`.

```toml
[dev-dependencies]
tokio-test = "0.4"
wiremock = "0.6"
tempfile = "3"
serial_test = "3"
assert_cmd = "=2.1.2"  # Pin: 2.1.3+ usa edition2024
predicates = "3"
```

### OBRIGATÓRIO — exclude

DEVE declarar `exclude` para proteger o tarball do crates.io (ver Seção 11).

```toml
exclude = [
    ".env",
    ".serena/",
    "logs/",
    "CLAUDE.md",
    "docs_rules/",
    "como_usa.md",
]
```

### OBRIGATÓRIO — [package.metadata.docs.rs]

DEVE declarar este bloco para que o docs.rs compile com todas as features e flags corretas.

```toml
[package.metadata.docs.rs]
all-features = true
rustdoc-args = ["--cfg", "docsrs"]
```

Ver `Cargo.toml` completo para referência.

---

## 9. Integração com docs.rs

**Princípio absoluto**: a documentação pública DEVE ser impecável. DEVE compilar sem warnings de rustdoc. DEVE ter exemplos executáveis (doctests) para funções públicas principais.

### 9.1 — Configuração em Cargo.toml

DEVE declarar `[package.metadata.docs.rs]` conforme Seção 8. DEVE ter `documentation = "https://docs.rs/<nome-da-crate>"` em `[package]`.

### 9.2 — Doc comments //! no lib.rs

DEVE iniciar `lib.rs` com bloco de doc comments `//!` descrevendo o propósito da crate e uma tabela de módulos.

```rust
//! minha-crate-cli library crate.
//!
//! Expõe a hierarquia pública de módulos e o entry point [`run`].
//!
//! # Visão geral dos módulos
//!
//! | Módulo | Responsabilidade |
//! |---|---|
//! | [`errors`] | Tipos de erro estruturados |
//! | [`i18n`] | i18n bilíngue (EN/PT) |
//! | [`storage`] | Persistência XDG, hierarquia de chaves |
//! | [`api`] | Cliente HTTP, retry, chamadas de API |
//! | [`output`] | Output terminal — ÚNICO módulo com `println!` |
//! | [`cli`] | Structs Clap, dispatchers de subcomandos |
```

Ver `src/lib.rs:1-16` para implementação real.

### 9.3 — Doc comments //! em cada módulo público

DEVE incluir bloco `//!` no topo de CADA módulo público (`api.rs`, `storage.rs`, etc.) descrevendo a responsabilidade.

### 9.4 — Exemplos compiláveis via doctest

DEVE incluir exemplos em blocos ` ```rust ` que compilam. DEVE adicionar `# use crate::...;` para imports necessários nos exemplos.

```rust
/// Mascara um valor de API key para exibição segura.
///
/// # Exemplo
///
/// ```rust
/// # use context7_cli::storage::mascarar_chave;
/// let mascarada = mascarar_chave("ctx7sk-abcdefghijkl-xyz9");
/// assert!(mascarada.contains("..."));
/// ```
pub fn mascarar_chave(chave: &str) -> String { ... }
```

### 9.5 — Badge docs.rs no README

DEVE incluir badge de status do docs.rs no topo do README:

```markdown
[![docs.rs](https://img.shields.io/docsrs/minha-crate-cli)](https://docs.rs/minha-crate-cli)
```

### 9.6 — Validação obrigatória

DEVE executar `cargo doc --no-deps --all-features` com `RUSTDOCFLAGS=-D warnings` no CI. ZERO warnings são tolerados.

```yaml
- name: Documentação (cargo doc)
  run: cargo doc --no-deps --all-features
  env:
    RUSTDOCFLAGS: -D warnings
```

### 9.7 — Build condicional com #[cfg(docsrs)]

DEVE usar `#[cfg(docsrs)]` para ativar imports ou impls que existem apenas na build do docs.rs.

```rust
#[cfg(docsrs)]
use some_optional_dep::Trait;
```

### PROIBIDO — docs.rs

- JAMAIS omita `[package.metadata.docs.rs]` em Cargo.toml
- JAMAIS aceite `cargo doc` com warnings — trate como erro de compilação
- JAMAIS escreva exemplos em doctest que não compilam
- JAMAIS aponte `documentation` para URL que não existe

---

## 10. `.gitignore` — Blindagem do Repositório

**Princípio absoluto**: o `.gitignore` DEVE proteger o repositório de 5 categorias: artefatos de build, logs, secrets, privado de desenvolvimento, e arquivos de IDE.

### OBRIGATÓRIO — Template mínimo

```gitignore
# Artefatos de build
target/

# Logs
logs/

# Secrets (NUNCA commitar)
.env

# Cargo.lock: EXCEÇÃO para binários — DEVE ser commitado
*.lock
!Cargo.lock

# Cobertura LLVM
*.profraw
*.profdata

# Privado — ferramentas de desenvolvimento, NUNCA no GitHub
.serena/
CLAUDE.md
docs_rules/
.claude/
```

Ver `.gitignore` completo do projeto para referência exata.

### OBRIGATÓRIO — Regra da exceção !Cargo.lock

DEVE commitar `Cargo.lock` para BINÁRIOS CLI. JAMAIS ignore `Cargo.lock` em binários — ele garante builds reproduzíveis e previne surpresas de dependência em produção.

DEVE ignorar `Cargo.lock` APENAS para bibliotecas puras (crates publicadas como `[lib]` sem `[[bin]]`).

### OBRIGATÓRIO — Categorias SEMPRE presentes

| Categoria       | Exemplos                                           | Motivo                                    |
| --------------- | -------------------------------------------------- | ----------------------------------------- |
| Build artifacts | `target/`                                          | Gigabytes de arquivos compilados          |
| Logs            | `logs/`, `*.log`                                   | Conteúdo sensível, gerado automaticamente |
| Secrets         | `.env`, `*.pem`, `*.key`                           | JAMAIS no git                             |
| Privado dev     | `.serena/`, `.claude/`, `CLAUDE.md`, `docs_rules/` | Contexto de IA, regras internas           |
| IDE             | `.idea/`, `.vscode/`, `*.swp`                      | Configurações locais                      |

### PROIBIDO — .gitignore

- JAMAIS commite `.env` — SEMPRE está em `.gitignore`
- JAMAIS commite `logs/` — gerado em runtime
- JAMAIS commite `.serena/` — contexto privado do Serena MCP
- JAMAIS commite `CLAUDE.md` — instruções internas de agente de IA
- JAMAIS commite `docs_rules/` — regras internas do projeto

---

## 11. `Cargo.toml [exclude]` — Blindagem do Pacote Publicado

**Princípio absoluto**: `.gitignore` protege o repositório GitHub. `[exclude]` protege o tarball do crates.io. São COMPLEMENTARES e AMBOS são OBRIGATÓRIOS.

### Diferença crítica

| Mecanismo    | Protege              | Quando importa                      |
| ------------ | -------------------- | ----------------------------------- |
| `.gitignore` | Commits no GitHub    | `git add`, `git commit`, `git push` |
| `[exclude]`  | Tarball no crates.io | `cargo publish`, `cargo package`    |

Um arquivo ignorado pelo `.gitignore` MAS não listado em `[exclude]` PODE vazar no tarball do crates.io se existir localmente.

### OBRIGATÓRIO — Lista mínima de exclude

```toml
[package]
exclude = [
    ".env",           # secrets
    ".serena/",       # contexto privado Serena MCP
    "logs/",          # logs de runtime
    "CLAUDE.md",      # instruções de agente de IA
    "docs_rules/",    # regras internas
    "como_usa.md",    # documentação interna
    ".claude/",       # memória de agente
]
```

Ver `Cargo.toml:14-21` para lista completa em uso.

### OBRIGATÓRIO — Verificação antes de publish

DEVE executar `cargo package --list` para verificar exatamente quais arquivos serão incluídos no tarball ANTES de publicar.

```bash
cargo package --list
# Verificar que .env, .serena/, CLAUDE.md NÃO aparecem na lista
```

### PROIBIDO — exclude

- JAMAIS publique sem executar `cargo package --list`
- JAMAIS omita `.env` do `exclude` — vazamento de secrets no crates.io é irreversível
- JAMAIS omita `logs/` do `exclude` — logs podem conter tokens e dados sensíveis
- JAMAIS confunda `.gitignore` com `[exclude]` — AMBOS são necessários

---

## 12. Pastas Ignoradas — Convenção de Privacidade

**Princípio absoluto**: certas pastas contêm contexto privado de desenvolvimento que JAMAIS deve aparecer em repositório público ou tarball de crate. Esta seção define o contrato de privacidade do projeto.

### OBRIGATÓRIO — Pastas privadas e seus propósitos

| Pasta / Arquivo | Propósito                                              | Proteções obrigatórias     |
| --------------- | ------------------------------------------------------ | -------------------------- |
| `.serena/`      | Memória e índices do Serena MCP (code intelligence)    | `.gitignore` + `[exclude]` |
| `docs_rules/`   | Regras internas do projeto (CLAUDE.md das regras Rust) | `.gitignore` + `[exclude]` |
| `.claude/`      | Memória persistente de sessões de agente IA            | `.gitignore` + `[exclude]` |
| `CLAUDE.md`     | Instruções internas para agentes de IA (este projeto)  | `.gitignore` + `[exclude]` |
| `logs/`         | Logs de runtime com possível conteúdo sensível         | `.gitignore` + `[exclude]` |
| `.env`          | Chaves de API e secrets locais                         | `.gitignore` + `[exclude]` |

### OBRIGATÓRIO — Dupla proteção

CADA item acima DEVE aparecer em AMBOS:

```
1. .gitignore  →  protege o repositório GitHub
2. Cargo.toml [exclude]  →  protege o tarball crates.io
```

JAMAIS proteja apenas um dos dois. Um arquivo ignorado pelo git mas não no `[exclude]` pode vazar no tarball se existir localmente no momento do `cargo publish`.

### PROIBIDO — Convenção de privacidade

- JAMAIS commite `.serena/` — contém índices e anotações de codebase privadas
- JAMAIS commite `CLAUDE.md` — contém instruções de sistema para agentes de IA
- JAMAIS commite `docs_rules/` — contém regras internas que não devem ser públicas
- JAMAIS commite `.claude/` — contém memória de sessões de agente

---

## 13. Anti-Vazamento de Secrets — 3 Camadas

**Princípio absoluto**: secrets JAMAIS aparecem em código, commits, logs ou tarball. A proteção é em 3 camadas independentes que se complementam. Se uma falha, as outras ainda protegem.

### Camada 1 — `.gitignore`

DEVE incluir TODOS os arquivos de secrets na lista (ver Seção 10):

```gitignore
.env
*.pem
*.key
*.p12
*.pfx
```

### Camada 2 — `Cargo.toml [exclude]`

DEVE excluir os mesmos arquivos do tarball (ver Seção 11):

```toml
exclude = [".env", ".serena/", "logs/", ...]
```

### Camada 3 — `deny.toml` para supply chain

DEVE usar `cargo-deny` para validar que nenhuma dependência transitiva tem vulnerabilidade conhecida, licença incompatível, ou origem desconhecida.

```toml
# deny.toml — configuração completa

[advisories]
db-urls = ["https://github.com/rustsec/advisory-db"]
unmaintained = "workspace"
ignore = []

[licenses]
allow = [
    "MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception",
    "Unlicense", "BSD-2-Clause", "BSD-3-Clause",
    "ISC", "Unicode-3.0", "MPL-2.0", "CDLA-Permissive-2.0",
]
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
wildcards = "allow"
deny = []

[sources]
unknown-registry = "deny"     # PROIBIDO registries não conhecidos
unknown-git = "deny"           # PROIBIDO git deps desconhecidos
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

Ver `deny.toml` completo para referência.

### OBRIGATÓRIO — Newtype ChaveApi com Zeroize

DEVE criar newtype `ChaveApi(String)` com `#[derive(Zeroize, ZeroizeOnDrop)]` para limpeza segura da memória ao drop. JAMAIS armazene chaves de API como `String` pura — use SEMPRE `ChaveApi` em TODA a cadeia (storage → api → cli).

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct ChaveApi(String);

impl ChaveApi {
    pub fn new(valor: String) -> Self {
        Self(valor)
    }

    pub fn valor(&self) -> &str {
        &self.0
    }
}

// Debug SEMPRE mascara — JAMAIS exibe valor completo
impl std::fmt::Debug for ChaveApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChaveApi({})", mascarar_chave(&self.0))
    }
}

// Display SEMPRE mascara — JAMAIS exibe valor completo
impl std::fmt::Display for ChaveApi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", mascarar_chave(&self.0))
    }
}
```

DEVE usar `ChaveApi` em TODA a cadeia de chaves:

- `storage.rs`: armazenar e carregar como `Vec<ChaveApi>`
- `api.rs`: receber `ChaveApi` e usar `.valor()` para header Authorization
- `cli.rs`: passar `ChaveApi` entre subcomandos
- Quando `ChaveApi` sai de escopo, `ZeroizeOnDrop` limpa a memória automaticamente
- JAMAIS converta `ChaveApi` para `String` sem necessidade — use `.valor()` para leitura temporária

### Complementar — Auditoria CI obrigatória

DEVE executar no CI:

```yaml
- name: Auditar vulnerabilidades (cargo audit)
  run: cargo audit

- name: Supply chain (cargo deny)
  run: cargo deny check advisories licenses bans sources
```

Ver `.github/workflows/ci.yml` jobs `security-audit` e `supply-chain`.

### Secrets em GitHub Actions

DEVE armazenar secrets em **GitHub Secrets** (Settings → Secrets → Actions). JAMAIS hardcode tokens em workflows. JAMAIS exiba secrets em logs de CI (GitHub Actions mascara automaticamente secrets registrados).

```yaml
# CORRETO — via GitHub Secrets
env:
  CARGO_REGISTRY_TOKEN: ${{ secrets.CRATES_IO_TOKEN }}

# ERRADO — JAMAIS hardcode
env:
  CARGO_REGISTRY_TOKEN: "real-token-aqui"  # PROIBIDO
```

Secrets necessários para o fluxo de release:

| Secret               | Uso                                |
| -------------------- | ---------------------------------- |
| `CRATES_IO_TOKEN`    | `cargo publish` no crates.io       |
| `APPLE_TEAM_ID`      | Assinatura Developer ID (opcional) |
| `APPLE_ID`           | Notarização Apple (opcional)       |
| `APPLE_APP_PASSWORD` | Notarização Apple (opcional)       |

---

## 14. Fluxo de Publicação GitHub Actions + crates.io

**Princípio absoluto**: a publicação JAMAIS acontece manualmente. SEMPRE via GitHub Actions com tag `v*`. O job de publicação no crates.io JAMAIS roda sem que os jobs de build e validação tenham passado.

### 14.1 — Arquitetura de 2 Workflows

DEVE ter exatamente 2 workflows:

| Workflow      | Trigger                                               | Propósito          |
| ------------- | ----------------------------------------------------- | ------------------ |
| `ci.yml`      | `push:main`, `pull_request:main`, `workflow_dispatch` | Validação contínua |
| `release.yml` | `push:tags:v*`, `workflow_dispatch (dry_run)`         | Build + publicação |

### 14.2 — Trigger de Release

DEVE configurar dois triggers em `release.yml`: `push: tags: ['v*']` para release real, e `workflow_dispatch` com input `dry_run: boolean` (default `true`) para testar o pipeline sem publicar. DEVE usar `dry_run: true` antes da primeira publicação real. Ver `.github/workflows/release.yml:1-16`.

### 14.3 — Ordem obrigatória de jobs

DEVE seguir exatamente esta ordem de dependências (`needs:`):

```
validate
    ↓
build-matrix (7 targets em paralelo)
    ↓
macos-universal (lipo arm64 + x86_64)
build-flatpak (paralelo com macos-universal)
build-snap (paralelo com macos-universal)
    ↓
publish-github-release
    ↓
publish-crates-io
```

### 14.4 — `cargo publish --dry-run` obrigatório

DEVE executar `cargo publish --dry-run` no job `validate` ANTES de qualquer build real. Isso detecta erros de metadados (versão duplicada, campos ausentes) sem consumir recursos de build.

```yaml
- name: Dry-run de publicação
  run: cargo publish --dry-run
```

Ver `.github/workflows/release.yml` job `validate`.

### 14.5 — 7 targets de build na matrix

DEVE compilar para exatamente estes 7 targets:

| OS             | Target                        | Formato   | Toolchain        |
| -------------- | ----------------------------- | --------- | ---------------- |
| ubuntu-latest  | `x86_64-unknown-linux-gnu`    | `.tar.gz` | nativa           |
| ubuntu-latest  | `x86_64-unknown-linux-musl`   | `.tar.gz` | `musl-tools`     |
| ubuntu-latest  | `aarch64-unknown-linux-gnu`   | `.tar.gz` | `cross`          |
| ubuntu-latest  | `aarch64-unknown-linux-musl`  | `.tar.gz` | `cross`          |
| macos-latest   | `aarch64-apple-darwin`        | `.tar.gz` | nativa           |
| macos-latest   | `x86_64-apple-darwin`         | `.tar.gz` | nativa           |
| windows-latest | `x86_64-pc-windows-msvc`      | `.zip`    | nativa           |

O target musl x86_64 REQUER `sudo apt-get install -y musl-tools` antes do build. Os targets ARM Linux DEVEM usar `cross` para cross-compilation:

```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-musl
```

### 14.6 — Universal Binary macOS via lipo

DEVE criar Universal Binary combinando `aarch64-apple-darwin` + `x86_64-apple-darwin`:

```bash
lipo -create \
    artifacts/aarch64/context7 \
    artifacts/x86_64/context7 \
    -output context7-universal
lipo -info context7-universal  # verificar que contém ambas as arquiteturas
```

DEVE assinar o binário universal (adhoc se sem certificado Developer ID):

```bash
codesign --sign - --force context7-universal  # assinatura adhoc
```

Ver `.github/workflows/release.yml` job `macos-universal` para lógica completa de assinatura condicional.

### 14.7 — softprops/action-gh-release@v2

DEVE usar `softprops/action-gh-release@v2` para criar o GitHub Release. DEVE extrair versão via `rg -m1 -o '^version\s*=\s*"(.+)"' Cargo.toml -r '$1'` e exportar para `$GITHUB_ENV`. DEVE incluir `files: artifacts/**/*.{tar.gz,zip,flatpak,snap}`, `generate_release_notes: false` e `fail_on_unmatched_files: false`. Ver `.github/workflows/release.yml` job `publish-github-release`.

### 14.8 — `always()` nos jobs de publicação

DEVE usar `if: always() && needs.X.result == 'success'` em jobs de publicação. Isso tolera falhas não-críticas de Flatpak/Snap sem bloquear o release principal. Ver `.github/workflows/release.yml` job `publish-github-release`.

### 14.9 — Checklist pré-tag obrigatório

DEVE verificar TODOS os itens antes de criar a tag `vX.Y.Z`:

- [ ] `CHANGELOG.md` atualizado com a versão e data
- [ ] `README.md` reflete novos comandos/flags se houver
- [ ] `version = "X.Y.Z"` bumped em `Cargo.toml`
- [ ] `cargo check` limpo (ZERO erros)
- [ ] `cargo clippy -- -D warnings` limpo (ZERO warnings)
- [ ] `cargo test` passando (ZERO falhando)
- [ ] `cargo publish --dry-run` passou sem erros

### PROIBIDO — Publicação

- JAMAIS crie tag sem completar o checklist 14.9
- JAMAIS hardcode `CRATES_IO_TOKEN` no workflow — SEMPRE via GitHub Secrets
- JAMAIS use `cargo publish` manualmente sem dry-run prévio
- JAMAIS pule o job `validate` com `force-push` de tag

---

## 15. Copywriting do README — Estratégia AIDA Bilíngue

**Princípio absoluto**: o README é o único artefato de marketing do projeto no crates.io. DEVE converter visitante em usuário em ≤30 segundos. JAMAIS escreva README que não responda "O que é?", "Por que usar?" e "Como começo?" nos primeiros 100 linhas.

### 15.1 — Badge Cluster no Topo

DEVE incluir exatamente estes 4 badges na primeira linha após o título:

```markdown
[![Crates.io](https://img.shields.io/crates/v/minha-crate-cli)](https://crates.io/crates/minha-crate-cli)
[![docs.rs](https://img.shields.io/docsrs/minha-crate-cli)](https://docs.rs/minha-crate-cli)
[![CI](https://github.com/user/repo/actions/workflows/ci.yml/badge.svg)](https://github.com/user/repo/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/user/repo#license)
```

Ver `README.md:3-6` para badges em uso.

### 15.2 — Hero Tagline Obrigatória

DEVE ter UMA linha de tagline em blockquote com o value proposition completo em ≤15 palavras.

```markdown
> Search any library's docs from your terminal — one binary, zero runtime, instant results.
```

Ver `README.md:11` para tagline real.

### 15.3 — Seção "What is it?" com 6 bullets

DEVE listar os 6 atributos técnicos mais relevantes em bullets. Cada bullet: **negrito** para o atributo, hífen, descrição de UMA linha. Ver `README.md:17-23` para bullets reais: Single binary, XDG-compliant storage, Multi-key rotation, Bilingual UI, Privacy-first, Structured output.

### 15.4 — Seção "Why?" com diferencial explícito

DEVE responder "por que não usar alternativa X?" em 3-4 bullets: zero context switching, pipe-friendly (`--text` para LLMs), works over SSH, CI/CD native. Ver `README.md:26-31`.

### 15.5 — Quick Start em 30 segundos

DEVE ter seção com ≤4 comandos (install → configure → use) e output esperado para validação visual. Ver `README.md:28-56` para Quick Start real com output esperado.

### 15.6 — Tabelas de Comandos por família

DEVE organizar comandos em tabelas separadas por subcomando (library, docs, keys). JAMAIS liste todos os comandos em uma única tabela amorfa. Ver `README.md:58-94`.

### 15.7 — Tabelas de Environment Variables e Output Formats

DEVE documentar TODAS as env vars em tabela com 3 colunas: Variável, Descrição, Exemplo. DEVE incluir `CRATE_API_KEYS`, `CRATE_HOME`, `CRATE_LANG`, `RUST_LOG`, `NO_COLOR`, `CLICOLOR_FORCE`. Ver `README.md:116-121` para tabela real. DEVE respeitar `NO_COLOR` (spec no-color.org) e `CLICOLOR_FORCE` como env vars padrão da indústria.

### 15.8 — Integration Patterns com exemplos pipeable

DEVE incluir 4-6 exemplos de uso em pipelines: pipe para LLM (`--text`), extração de ID com `jaq`, uso em CI/CD via env var. Ver `README.md:125-137` para integration patterns reais.

### 15.9 — Quick Reference como tabela de lookup

DEVE incluir tabela de referência rápida com operações mais comuns (library, docs, keys add/list/remove). Ver `README.md:139-148`.

### 15.10 — Exit codes documentados (mapeamento BSD sysexits)

DEVE documentar TODOS os exit codes com mapeamento BSD:

| Código | Nome BSD       | Significado                             |
| ------ | -------------- | --------------------------------------- |
| 0      | —              | Sucesso                                 |
| 1      | —              | Erro genérico de runtime                |
| 2      | —              | Uso incorreto de CLI (argumento errado) |
| 65     | EX_DATAERR     | Dados de input inválidos                |
| 66     | EX_NOINPUT     | Recurso não encontrado                  |
| 69     | EX_UNAVAILABLE | Serviço indisponível após retry         |
| 74     | EX_IOERR       | Erro de I/O ou rede                     |
| 77     | EX_NOPERM      | Permissão/autenticação negada           |
| 130    | —              | Terminação por SIGINT (Ctrl+C)          |

DEVE usar `std::process::ExitCode` ou constantes nomeadas para CADA código. JAMAIS use magic numbers diretamente no código.

### 15.10.1 — Signal Handling com graceful shutdown

DEVE implementar signal handling com `tokio::signal::ctrl_c()` + `tokio::select!` para graceful shutdown. DEVE retornar exit code 130 ao receber SIGINT (Ctrl+C). JAMAIS ignore sinais — o binário DEVE encerrar limpo ao receber Ctrl+C.

```rust
tokio::select! {
    resultado = executar_operacao() => resultado,
    _ = tokio::signal::ctrl_c() => {
        eprintln!("\n{}", t(Mensagem::InterrompidoPeloUsuario));
        std::process::exit(130);
    }
}
```

### 15.10.2 — Flags de output OBRIGATÓRIAS

DEVE implementar TODAS estas flags globais:

| Flag          | Env var equivalente | Comportamento                                        |
| ------------- | ------------------- | ---------------------------------------------------- |
| `--no-color`  | `NO_COLOR=1`        | Desabilita cores ANSI no output                      |
| `--plain`     | `CLICOLOR_FORCE=0`  | Output sem formatação visual (sem cores, sem Unicode) |
| `--verbose`   | `RUST_LOG=debug`    | Ativa output detalhado de debug                      |
| `--quiet`     | —                   | Suprime output exceto erros                          |

DEVE respeitar `NO_COLOR` conforme spec https://no-color.org/ — se a variável existir (qualquer valor), desabilitar cores. DEVE respeitar `CLICOLOR_FORCE` para forçar ou desabilitar cores em pipelines.

### 15.10.3 — Fallback ASCII para símbolos Unicode

DEVE implementar fallback de símbolos Unicode para ASCII quando o terminal NÃO suportar Unicode. DEVE detectar via `NO_COLOR`, `TERM=dumb` ou `--plain`.

| Símbolo Unicode | Fallback ASCII | Uso                    |
| --------------- | -------------- | ---------------------- |
| `✓`             | `[OK]`         | Sucesso                |
| `✗`             | `[FAIL]`       | Falha                  |
| `→`             | `->`           | Seta de direção        |
| `•`             | `*`            | Bullet point           |
| `…`             | `...`          | Reticências            |

### 15.10.4 — Formato NDJSON como contrato para LLMs

DEVE suportar output em NDJSON (Newline-Delimited JSON) via `--json` para consumo por LLMs e pipelines automatizados. CADA linha de output NDJSON DEVE ser um objeto JSON completo e auto-contido. JAMAIS quebre um objeto JSON em múltiplas linhas quando `--json` estiver ativo.

```bash
context7 library react --json | jaq '.[0].id'
context7 docs /reactjs/react.dev --query "hooks" --json | jaq '.snippets[0]'
```

NDJSON é o CONTRATO de comunicação entre a CLI e qualquer LLM que consuma o output.

### 15.11 — Troubleshooting FAQ

DEVE incluir seção com 3-5 problemas comuns + resolução de 1 linha: "No API key configured", "401 Unauthorized", "binário não encontrado após install".

### 15.12 — Seção What's New

DEVE incluir link para `CHANGELOG.md`. JAMAIS escreva changelog inline no README.

### 15.13 — Estrutura Bilíngue EN + PT

DEVE organizar o README em duas seções espelhadas: `## English` primeiro, `## Português` depois. JAMAIS misture idiomas dentro de uma seção. JAMAIS use tradução automática — redação nativa. Ver `README.md` completo para estrutura bilíngue real.

### 15.14 — Licença no final

DEVE incluir seção de licença no final do README com texto da licença e links.

### 15.15 — Bilinguismo: seções espelhadas

DEVE manter as duas versões sincronizadas a cada atualização. JAMAIS atualize EN sem atualizar PT (e vice-versa).

### 15.16 — PROIBIDO — README

- JAMAIS use emojis excessivos — máximo 3 por seção, apenas se acrescentam valor semântico
- JAMAIS escreva "Este projeto demonstra..." ou "Ficamos felizes que..." — zero narrativa
- JAMAIS deixe seção "Installation" sem comando `cargo install`
- JAMAIS omita a seção de Environment Variables
- JAMAIS esqueça a tabela de comandos de gestão de configuração

---

## 16. Bloqueio de Co-authored-by — Claude + GitHub

**Princípio absoluto**: Co-authored-by de agentes de IA JAMAIS deve aparecer em commits no branch `main`. A proteção é em 3 camadas independentes.

### 16.1 — Regra global em ~/.claude/CLAUDE.md

DEVE incluir esta diretiva no arquivo global de instruções de agentes de IA:

```markdown
## Regras do Projeto
- NUNCA adicionar Co-authored-by em commits
```

Esta regra impede que Claude Code, Cursor, GitHub Copilot e outros agentes que respeitam `CLAUDE.md` adicionem o trailer automaticamente.

### 16.2 — Regra por projeto em CLAUDE.md

DEVE reforçar a mesma regra no `CLAUDE.md` do projeto:

```markdown
# Papel, Identidade e Missão
- NUNCA adicionar Co-authored-by em commits
```

### 16.3 — Hook CI commit-check

DEVE implementar job `commit-check` em `.github/workflows/ci.yml` que roda em `pull_request`:

```yaml
commit-check:
  name: Verificar trailers de commit (anti-bot-coauthor)
  runs-on: ubuntu-latest
  if: github.event_name == 'pull_request'
  steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    - name: Bloquear Co-authored-by de bots
      shell: bash
      run: |
        BOTS="dep-bot\[bot\]|renovate\[bot\]|github-actions\[bot\]"
        COMMITS=$(git log origin/${{ github.base_ref }}..HEAD --format="%H %s")
        FOUND=0
        while IFS= read -r line; do
          SHA=${line%% *}
          MSG=$(git log -1 --format="%B" "$SHA")
          if echo "$MSG" | rg -qi "Co-authored-by:.*($BOTS)"; then
            echo "ERRO: commit $SHA contém Co-authored-by de bot:"
            echo "$MSG" | rg -i "Co-authored-by:"
            FOUND=1
          fi
        done <<< "$COMMITS"
        if [ "$FOUND" -eq 1 ]; then
          echo "Use 'Squash and merge' com 'Use pull request title and description'"
          exit 1
        fi
        echo "OK: nenhum trailer Co-authored-by de bot encontrado."
```

Ver `.github/workflows/ci.yml:23-54` para implementação real.

### ALERTA — GAP IDENTIFICADO NO CI ATUAL DO context7-cli v0.5.0

A regex atual no CI é:

```
Co-authored-by:.*(dep-bot\[bot\]|renovate\[bot\]|github-actions\[bot\])
```

**GAP**: esta regex SOMENTE captura trailers onde o Co-author termina com o sufixo `[bot]`. Ela NÃO captura:

- `Co-authored-by: Claude Opus 4.6 <noreply@anthropic.com>`
- `Co-authored-by: GPT-4 <noreply@openai.com>`
- `Co-authored-by: GitHub Copilot <copilot@github.com>`
- `Co-authored-by: Cursor AI <noreply@cursor.sh>`
- `Co-authored-by: Gemini <noreply@google.com>`

Nenhum destes termina com `[bot]` — o padrão atual os deixa passar silenciosamente.

### 16.4 — Regex Estendida Proposta

DEVE atualizar a regex para cobrir agentes de IA modernos:

```bash
BOTS="dep-bot\[bot\]|renovate\[bot\]|github-actions\[bot\]|Claude|Opus|Sonnet|Haiku|GPT-|Copilot|Cursor|Gemini|Anthropic|OpenAI"
```

Esta regex captura:

| Padrão                | Agente bloqueado              |
| --------------------- | ----------------------------- |
| `dep-bot[bot]`     | Automated dependency bot      |
| `renovate[bot]`       | Renovate Bot                  |
| `github-actions[bot]` | GitHub Actions                |
| `Claude`              | Claude (Opus, Sonnet, Haiku)  |
| `Opus`                | Claude Opus especificamente   |
| `Sonnet`              | Claude Sonnet especificamente |
| `Haiku`               | Claude Haiku especificamente  |
| `GPT-`                | GPT-4, GPT-3.5, GPT-4o        |
| `Copilot`             | GitHub Copilot                |
| `Cursor`              | Cursor AI                     |
| `Gemini`              | Google Gemini                 |
| `Anthropic`           | Qualquer agente Anthropic     |
| `OpenAI`              | Qualquer agente OpenAI        |

### 16.5 — PR Template com Squash and Merge

DEVE configurar o repositório GitHub para:

- **Squash and merge**: único método de merge permitido
- **Pull request title and description**: como mensagem do commit de squash
- Isso elimina automaticamente trailers Co-authored-by dos commits squashados

### 16.6 — Git config local

DEVE configurar `user.name` e `user.email` reais no repositório:

```bash
git config user.name "Nome Real"
git config user.email "email@real.com"
```

JAMAIS use `user.name` genérico ou email de agente de IA como identidade de committer.

### 16.7 — Squash de bots externos

Para PRs automáticos de bots de dependência ou Renovate, DEVE usar "Squash and merge" com "Use pull request title and description" — elimina o trailer `Co-authored-by: dep-bot[bot]` da mensagem final.

---

## 17. Checklist Final de Replicação

**Princípio absoluto**: JAMAIS declare o projeto "pronto para publicação" sem marcar TODOS os 30 itens como ✅. Um item ☐ é um bloqueador de publicação.

| #   | Item                                                                                     | §Ref   |
| --- | ---------------------------------------------------------------------------------------- | ------ |
| 1   | ☐ `src/` tem exatamente os 9 módulos com responsabilidades isoladas                      | §2     |
| 2   | ☐ `output.rs` é o ÚNICO módulo com `println!`                                            | §2     |
| 3   | ☐ `lib.rs` tem bloco `//!` com tabela de módulos                                         | §2, §9 |
| 4   | ☐ `main.rs` contém APENAS entry point + `GuardaLog`                                      | §2     |
| 5   | ☐ `tests/` tem arquivo separado por domínio de teste                                     | §2, §7 |
| 6   | ☐ `rust-version = "X.Y"` declarado em `Cargo.toml`                                       | §3     |
| 7   | ☐ Job `msrv` em `ci.yml` verifica `rust-version`                                         | §3     |
| 8   | ☐ Pins `=x.y.z` com comentário de motivo para crates problemáticas                       | §3     |
| 9   | ☐ `cargo check` limpo no MSRV declarado                                                  | §3     |
| 10  | ☐ Hierarquia de 4 camadas implementada na ordem correta                                  | §4.1   |
| 11  | ☐ `directories::ProjectDirs` usado para TODOS os paths de config                         | §4.2   |
| 12  | ☐ Override `<CRATE>_HOME` implementado com anti-path-traversal                           | §4.3   |
| 13  | ☐ `aplicar_permissoes_600` chamado após cada escrita de config                           | §4.4   |
| 14  | ☐ `schema_version: u32` no struct de config persistida                                   | §4.5   |
| 15  | ☐ Timestamps RFC 3339 via `chrono::Utc::now().to_rfc3339()`                              | §4.6   |
| 16  | ☐ Mascaramento aplicado em TODA exibição de valores sensíveis                            | §4.7   |
| 17  | ☐ Deduplicação automática no `add`                                                       | §4.8   |
| 18  | ☐ Enum `Mensagem` como ÚNICA fonte de strings de UI                                      | §6.2   |
| 19  | ☐ `match` exaustivo em `en()` e `pt()` — compiler garante cobertura                      | §6.3   |
| 20  | ☐ `OnceLock<Idioma>` para thread-safety sem `Mutex`                                      | §6.5   |
| 21  | ☐ Precedência de idioma: `--lang` > env > sys-locale > EN                                | §6.4   |
| 22  | ☐ Todo teste E2E usa `TempDir` + `env_clear()` + `<CRATE>_HOME`                          | §7.3   |
| 23  | ☐ Testes com env vars têm `#[serial]`                                                    | §7.4   |
| 24  | ☐ ZERO chamadas HTTP reais em testes — `wiremock` obrigatório                            | §7.5   |
| 25  | ☐ Cobertura ≥80% via `cargo llvm-cov`                                                    | §7.6   |
| 26  | ☐ Matrix CI em ubuntu + macos + windows                                                  | §7.7   |
| 27  | ☐ `.gitignore` cobre: `target/`, `logs/`, `.env`, `.serena/`, `CLAUDE.md`                | §10    |
| 28  | ☐ `[exclude]` em `Cargo.toml` espelha `.gitignore` para secrets                          | §11    |
| 29  | ☐ `deny.toml` configurado com advisories + licenses + bans + sources                     | §13    |
| 30  | ☐ Regex `commit-check` atualizada para incluir padrões LLM (`Claude`, `GPT-`, `Copilot`) | §16    |

### Validação Final Obrigatória (executar na ordem)

```bash
cargo check --all-targets
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all --check
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
cargo test --all-features
cargo llvm-cov --text
cargo audit
cargo deny check advisories licenses bans sources
cargo publish --dry-run
cargo package --list
```

JAMAIS publique sem que os 10 comandos acima passem sem erros.

---

*Referência: context7-cli v0.5.0 — `/home/comandoaguiar/Dropbox/dev/rust/linux/cli_context7`*
*Fontes primárias: `src/storage.rs`, `src/i18n.rs`, `src/lib.rs`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`, `Cargo.toml`, `deny.toml`, `.gitignore`*
