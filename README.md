# context7-cli

[![Crates.io](https://img.shields.io/crates/v/context7-cli)](https://crates.io/crates/context7-cli)
[![docs.rs](https://img.shields.io/docsrs/context7-cli)](https://docs.rs/context7-cli)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](https://github.com/danilo-aguiar-br/context7-cli#license)


## English

> Search any library's docs from your terminal — one binary, zero runtime, instant results.

### What is context7-cli?

context7-cli queries the [Context7](https://context7.com) REST API directly from your shell. No Node, no Python, no daemon. One static binary that installs in seconds and works identically on Linux, macOS, and Windows.

Key properties:
- Single binary — zero runtime dependencies, ships as a static executable
- XDG-compliant storage — API keys stored in `~/.config/context7/config.toml` with `chmod 600`
- Multi-key rotation — automatic shuffle with exponential backoff retry (500ms → 1s → 2s, up to 5 attempts)
- Bilingual UI — English and Brazilian Portuguese output, auto-detected from system locale
- Privacy-first — no telemetry, no analytics; keys are masked in all logs and list output
- Secure key handling — API keys wrapped in a secure newtype with automatic memory zeroing on drop (zeroize)
- Structured output — colored terminal, `--json` for pipes, `--text` for LLM context windows
- Graceful signal handling — Ctrl+C triggers proper cleanup and exits with code 130

### Why context7-cli?

- Zero context switching — stay in your terminal, no browser tabs needed
- Pipe-friendly — `--text` output feeds directly into LLMs, scripts, and pipelines
- Works over SSH — full functionality on headless servers and remote machines
- CI/CD native — set `CONTEXT7_API_KEYS` and use it in any automation workflow

### Quick Start (30 seconds)

```bash
# 1 — Install
cargo install context7-cli

# 2 — Add your API key (get one at https://context7.com)
context7 keys add ctx7sk-YOUR-KEY-HERE

# 3 — Search a library
context7 library react
```

Expected output:

```
Libraries found:
────────────────────────────────────────────────────────────
1. React (trust 10.0/10)
   /reactjs/react.dev
   React.dev is the official documentation website for React...
```

```bash
# 4 — Fetch documentation
context7 docs /tokio-rs/tokio --query "spawn_blocking" --text
```

> No Rust toolchain? Download a prebuilt binary from the [GitHub Releases page](https://github.com/danilo-aguiar-br/context7-cli/releases/latest). Available targets:
> - `x86_64-unknown-linux-gnu` (Linux x86_64 glibc)
> - `x86_64-unknown-linux-musl` (Linux x86_64 musl — Alpine containers)
> - `aarch64-unknown-linux-gnu` (ARM64 Linux glibc — Raspberry Pi 4+, AWS Graviton)
> - `aarch64-unknown-linux-musl` (ARM64 Linux musl — Alpine containers on ARM)
> - `x86_64-pc-windows-msvc` (Windows x86_64 zip)
> - macOS Universal Binary (arm64 + x86_64)

### Commands

#### Search Libraries

```bash
context7 library react
context7 lib axum "middleware"           # alias: lib, search
context7 search tokio "mpsc channel"    # optional semantic context
```

Aliases: `library`, `lib`, `search` are equivalent.

#### Fetch Documentation

```bash
context7 docs /reactjs/react.dev
context7 docs /tokio-rs/tokio --query "spawn_blocking"
context7 docs /tokio-rs/tokio -q "spawn_blocking" --text   # --text strips all markup
context7 doc  /rust-lang/rust --query "lifetimes" --json   # alias: doc, context
```

Use `--text` to pipe clean prose directly into an LLM context window. Use `--json` to process with `jaq` or scripts.

#### Key Management

| Command | Description |
|---------|-------------|
| `context7 keys add ctx7sk-...` | Add an API key |
| `context7 keys list` | List saved keys (masked) |
| `context7 keys list --json` | List keys as JSON array |
| `context7 keys remove 2` | Remove key by 1-based index |
| `context7 keys clear --yes` | Remove all keys |
| `context7 keys path` | Show config file path |
| `context7 keys export` | Export to `.env` format |
| `context7 keys import .env` | Import from `.env` file |

Key rotation is automatic — every request shuffles keys randomly.

### Output Formats

| Flag | Description | Best for |
|------|-------------|----------|
| (none) | Colored, human-readable | Terminal reading |
| `--json` | NDJSON (one JSON object per line) | Scripts, pipes, `jaq` |
| `--text` | Plain text, no markup (docs only) | LLM context windows |
| `--lang en\|pt` | Force UI language | Multilingual workflows |

When `--json` is active, output uses NDJSON format (one JSON object per line). Each line includes a `type` field (event category) and a `timestamp` field (ISO 8601). Example: `context7 library react --json | jq -c '.type'`

### Global Flags

| Flag | Description |
|------|-------------|
| `--no-color` | Disable colored output (also via `NO_COLOR` env var) |
| `--plain` | Plain text without ANSI (incompatible with `--json`) |
| `-v, --verbose` | Increase verbosity (`-v` info, `-vv` debug, `-vvv` trace) |
| `--quiet` | Suppress all output except errors |

### Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `CONTEXT7_API_KEYS` | Comma-separated keys (overrides config file) | `ctx7sk-key1,ctx7sk-key2` |
| `CONTEXT7_HOME` | Override XDG config directory | `$HOME/.dotfiles/context7` |
| `CONTEXT7_LANG` | Force UI language | `en` or `pt` |
| `RUST_LOG` | Log verbosity | `warn`, `info`, `debug`, `trace` |
| `NO_COLOR` | Any value disables colors (convention: no-color.org) | `1` |
| `CLICOLOR_FORCE` | Set to `1` to force colors in pipes | `1` |

Language auto-detect order: `--lang` flag → `CONTEXT7_LANG` → system locale → English.

### Integration Patterns

```bash
# Pipe documentation into an LLM prompt
context7 docs /tokio-rs/tokio -q "channels" --text | llm "explain buffered channels"

# Extract library IDs for scripting
context7 library axum --json | jaq '.[0].id'

# Search documentation with ripgrep
context7 docs /rust-lang/rust --text | rg "lifetime"

# Loop over multiple libraries
for lib in tokio axum serde; do
  context7 library "$lib" --json | jaq '.[0].id'
done

# CI/CD — no config file needed
CONTEXT7_API_KEYS="ctx7sk-key-01" context7 docs /tokio-rs/tokio -q "spawn" --text
```

### Quick Reference

| Operation | Command |
|-----------|---------|
| Search library | `context7 library NAME [CONTEXT]` |
| Fetch docs | `context7 docs ID [-q TEXT] [--text\|--json]` |
| Add key | `context7 keys add ctx7sk-...` |
| List keys | `context7 keys list [--json]` |
| Remove key | `context7 keys remove N` |
| Clear all keys | `context7 keys clear --yes` |
| Config path | `context7 keys path` |
| Export keys | `context7 keys export` |
| Import keys | `context7 keys import FILE` |
| Force language | `context7 --lang en\|pt COMMAND` |
| Shell completions | `context7 completions bash\|zsh\|fish\|powershell\|elvish` |

#### Shell Completions

```bash
# Bash
context7 completions bash > ~/.local/share/bash-completion/completions/context7

# Zsh (add ~/.zfunc to fpath in .zshrc first)
context7 completions zsh > ~/.zfunc/_context7

# Fish
context7 completions fish > ~/.config/fish/completions/context7.fish

# PowerShell (add to $PROFILE)
context7 completions powershell >> $PROFILE

# Elvish
context7 completions elvish > ~/.config/elvish/lib/context7.elv
```

### Exit Codes

| Code | Meaning | Example |
|------|---------|---------|
| `0` | Success | Successful search, key added, completions generated |
| `1` | Generic runtime error | No API keys configured, empty key, invalid index |
| `2` | Invalid CLI usage | Unknown subcommand, conflicting flags (`--text` + `--json`) |
| `65` | Invalid input data (EX_DATAERR) | Malformed API key, unparseable response |
| `66` | Resource not found (EX_NOINPUT) | Library or doc ID not found (API 404) |
| `69` | Service unavailable after retry (EX_UNAVAILABLE) | API down after all retry attempts exhausted |
| `74` | I/O or network error (EX_IOERR) | Connection refused, DNS failure, disk write error |
| `77` | Permission/auth denied (EX_NOPERM) | Invalid API key (401), rate limited (429) |
| `130` | Interrupted by Ctrl+C (SIGINT) | User pressed Ctrl+C during operation |

### Troubleshooting

`command not found: context7`
Add `~/.cargo/bin` to your `PATH`: `export PATH="$HOME/.cargo/bin:$PATH"`

"No API key configured"
Run `context7 keys add ctx7sk-YOUR-KEY` or set `CONTEXT7_API_KEYS` env var.

"Library not found" (404)
The library ID must come from `context7 library NAME`. Copy the `/org/repo` path from the search results, not from GitHub.

Rate limited (429)
The CLI retries automatically with exponential backoff (up to 5 attempts). Add a second key with `context7 keys add` to increase throughput.

macOS Gatekeeper blocks the binary
Run: `xattr -d com.apple.quarantine context7 && chmod +x context7`
Or install via `cargo install context7-cli` to bypass Gatekeeper entirely.

No colors in terminal
Use `--no-color` or `--plain` for plain output, or set `NO_COLOR=1`. Verify your terminal supports ANSI escape sequences.

`--text` and `--json` together give an error
These flags are mutually exclusive. Use one or the other: `context7 docs /org/repo --text` or `context7 docs /org/repo --json`.

`keys remove 0` gives "Invalid index"
Indices are 1-based. Run `context7 keys list` first to see the current indices, then use `context7 keys remove <N>` with N ≥ 1.

`context7 library` without arguments gives an error
The library name is required. Provide a search term: `context7 library react`

### What's New in v0.5.0

- NDJSON structured output — `--json` now emits one JSON object per line with `type` and `timestamp` fields for easy pipeline processing
- Global flags — `--no-color`, `--plain`, `-v/--verbose`, `--quiet` for full control over output behavior
- Expanded exit codes — sysexits-compatible codes (65, 66, 69, 74, 77, 130) for precise error classification in scripts
- Secure key zeroize — API keys wrapped in a secure newtype with automatic memory zeroing on drop
- ARM64 Linux builds — `aarch64-unknown-linux-gnu` and `aarch64-unknown-linux-musl` targets for Raspberry Pi 4+, AWS Graviton, and ARM Alpine containers
- Color control — `NO_COLOR` and `CLICOLOR_FORCE` environment variables supported (no-color.org convention)
- Graceful Ctrl+C — signal handler performs proper cleanup and exits with code 130

See [CHANGELOG.md](CHANGELOG.md) for the complete history.

### Full Documentation

- [HOW_TO_USE.md](docs/HOW_TO_USE.md) — step-by-step guide for Linux, macOS, and Windows
- [CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md) — Windows UTF-8, macOS Gatekeeper, Linux musl/Flatpak/Snap, NixOS, GNU Guix
- [context7-skill.md](docs/context7-skill.md) — LLM integration patterns
- [context7-llm-rules.md](docs/context7-llm-rules.md) — system prompt rules for AI assistants


## Português

> Pesquise a documentação de qualquer biblioteca no terminal — um binário, sem runtime, resultados instantâneos.

### O que é o context7-cli?

O context7-cli consulta a API REST do [Context7](https://context7.com) diretamente do seu shell. Sem Node, sem Python, sem daemon. Um único binário estático que instala em segundos e funciona de forma idêntica no Linux, macOS e Windows.

Propriedades principais:
- Binário único — zero dependências de runtime, distribuído como executável estático
- Armazenamento XDG — chaves de API salvas em `~/.config/context7/config.toml` com `chmod 600`
- Rotação multi-chave — shuffle automático com retry e backoff exponencial (500ms → 1s → 2s, até 5 tentativas)
- Interface bilíngue — saída em inglês e português brasileiro, detectado automaticamente pelo locale do sistema
- Privacidade em primeiro lugar — sem telemetria, sem analytics; chaves mascaradas em todos os logs e no output de listagem
- Chaves seguras com zeroize — chaves de API encapsuladas em newtype seguro com limpeza automática de memória ao sair de escopo
- Saída estruturada — colorido para terminal, `--json` para pipes, `--text` para janelas de contexto de LLM
- Tratamento gracioso de sinais — Ctrl+C executa limpeza adequada e encerra com código 130

### Por que o context7-cli?

- Zero troca de contexto — fique no terminal, sem abas de browser
- Amigável para pipes — saída `--text` alimenta LLMs, scripts e pipelines diretamente
- Funciona via SSH — funcionalidade completa em servidores headless e máquinas remotas
- Nativo para CI/CD — configure `CONTEXT7_API_KEYS` e use em qualquer workflow de automação

### Início Rápido (30 segundos)

```bash
# 1 — Instalar
cargo install context7-cli

# 2 — Adicionar sua chave de API (obtenha em https://context7.com)
context7 keys add ctx7sk-SUA-CHAVE-AQUI

# 3 — Buscar uma biblioteca
context7 library react
```

Saída esperada:

```
Bibliotecas encontradas:
────────────────────────────────────────────────────────────
1. React (confiança 10.0/10)
   /reactjs/react.dev
   React.dev é o site oficial de documentação do React...
```

```bash
# 4 — Buscar documentação
context7 docs /tokio-rs/tokio --query "spawn_blocking" --text
```

> Sem toolchain Rust? Baixe um binário pré-compilado na [página de Releases do GitHub](https://github.com/danilo-aguiar-br/context7-cli/releases/latest). Targets disponíveis:
> - `x86_64-unknown-linux-gnu` (Linux x86_64 glibc)
> - `x86_64-unknown-linux-musl` (Linux x86_64 musl — containers Alpine)
> - `aarch64-unknown-linux-gnu` (ARM64 Linux glibc — Raspberry Pi 4+, AWS Graviton)
> - `aarch64-unknown-linux-musl` (ARM64 Linux musl — containers Alpine em ARM)
> - `x86_64-pc-windows-msvc` (Windows x86_64 zip)
> - Universal Binary macOS (arm64 + x86_64)

### Comandos

#### Buscar Bibliotecas

```bash
context7 library react
context7 lib axum "middleware"          # alias: lib, search
context7 search tokio "canal mpsc"     # contexto semântico opcional
```

Aliases: `library`, `lib`, `search` são equivalentes.

#### Buscar Documentação

```bash
context7 docs /reactjs/react.dev
context7 docs /tokio-rs/tokio --query "spawn_blocking"
context7 docs /tokio-rs/tokio -q "spawn_blocking" --text   # --text remove toda marcação
context7 doc  /rust-lang/rust --query "lifetimes" --json   # alias: doc, context
```

Use `--text` para enviar prosa limpa diretamente para a janela de contexto de um LLM. Use `--json` para processar com `jaq` ou scripts.

#### Gerenciamento de Chaves

| Comando | Descrição |
|---------|-----------|
| `context7 keys add ctx7sk-...` | Adicionar uma chave de API |
| `context7 keys list` | Listar chaves salvas (mascaradas) |
| `context7 keys list --json` | Listar chaves como array JSON |
| `context7 keys remove 2` | Remover chave por índice 1-based |
| `context7 keys clear --yes` | Remover todas as chaves |
| `context7 keys path` | Exibir caminho do arquivo de configuração |
| `context7 keys export` | Exportar no formato `.env` |
| `context7 keys import .env` | Importar de arquivo `.env` |

A rotação de chaves é automática — cada requisição embaralha as chaves aleatoriamente.

### Formatos de Saída

| Flag | Descrição | Melhor para |
|------|-----------|-------------|
| (nenhuma) | Colorido, legível por humanos | Leitura no terminal |
| `--json` | NDJSON (um objeto JSON por linha) | Scripts, pipes, `jaq` |
| `--text` | Texto plano, sem marcação (apenas `docs`) | Janelas de contexto de LLM |
| `--lang en\|pt` | Forçar idioma da interface | Fluxos multilíngues |

Quando `--json` está ativo, a saída usa formato NDJSON (um objeto JSON por linha). Cada linha inclui um campo `type` (categoria do evento) e um campo `timestamp` (ISO 8601). Exemplo: `context7 library react --json | jq -c '.type'`

### Flags Globais

| Flag | Descrição |
|------|-----------|
| `--no-color` | Desabilitar saída colorida (também via variável `NO_COLOR`) |
| `--plain` | Texto plano sem ANSI (incompatível com `--json`) |
| `-v, --verbose` | Aumentar verbosidade (`-v` info, `-vv` debug, `-vvv` trace) |
| `--quiet` | Suprimir toda saída exceto erros |

### Variáveis de Ambiente

| Variável | Descrição | Exemplo |
|----------|-----------|---------|
| `CONTEXT7_API_KEYS` | Chaves separadas por vírgula (substitui o arquivo de configuração) | `ctx7sk-chave1,ctx7sk-chave2` |
| `CONTEXT7_HOME` | Override do diretório de configuração XDG | `$HOME/.dotfiles/context7` |
| `CONTEXT7_LANG` | Forçar idioma da interface | `en` ou `pt` |
| `RUST_LOG` | Verbosidade de log | `warn`, `info`, `debug`, `trace` |
| `NO_COLOR` | Qualquer valor desabilita cores (convenção: no-color.org) | `1` |
| `CLICOLOR_FORCE` | Definir como `1` para forçar cores em pipes | `1` |

Ordem de detecção de idioma: flag `--lang` → `CONTEXT7_LANG` → locale do sistema → inglês.

### Padrões de Integração

```bash
# Enviar documentação para um LLM
context7 docs /tokio-rs/tokio -q "channels" --text | llm "explique canais com buffer"

# Extrair IDs de biblioteca para scripts
context7 library axum --json | jaq '.[0].id'

# Buscar na documentação com ripgrep
context7 docs /rust-lang/rust --text | rg "lifetime"

# Iterar sobre múltiplas bibliotecas
for lib in tokio axum serde; do
  context7 library "$lib" --json | jaq '.[0].id'
done

# CI/CD — sem arquivo de configuração necessário
CONTEXT7_API_KEYS="ctx7sk-chave-01" context7 docs /tokio-rs/tokio -q "spawn" --text
```

### Referência Rápida

| Operação | Comando |
|----------|---------|
| Buscar biblioteca | `context7 library NOME [CONTEXTO]` |
| Buscar docs | `context7 docs ID [-q TEXTO] [--text\|--json]` |
| Adicionar chave | `context7 keys add ctx7sk-...` |
| Listar chaves | `context7 keys list [--json]` |
| Remover chave | `context7 keys remove N` |
| Limpar todas as chaves | `context7 keys clear --yes` |
| Caminho da configuração | `context7 keys path` |
| Exportar chaves | `context7 keys export` |
| Importar chaves | `context7 keys import ARQUIVO` |
| Forçar idioma | `context7 --lang en\|pt COMANDO` |
| Autocompletar | `context7 completions bash\|zsh\|fish\|powershell\|elvish` |

#### Autocompletar no Shell

```bash
# Bash
context7 completions bash > ~/.local/share/bash-completion/completions/context7

# Zsh (adicione ~/.zfunc ao fpath no .zshrc primeiro)
context7 completions zsh > ~/.zfunc/_context7

# Fish
context7 completions fish > ~/.config/fish/completions/context7.fish

# PowerShell (adicione ao $PROFILE)
context7 completions powershell >> $PROFILE

# Elvish
context7 completions elvish > ~/.config/elvish/lib/context7.elv
```

### Códigos de Saída

| Código | Significado | Exemplo |
|--------|-------------|---------|
| `0` | Sucesso | Busca realizada, chave adicionada, completions geradas |
| `1` | Erro genérico de runtime | Sem chaves configuradas, chave vazia, índice inválido |
| `2` | Uso inválido da CLI | Subcomando desconhecido, flags conflitantes (`--text` + `--json`) |
| `65` | Dados de entrada inválidos (EX_DATAERR) | Chave de API malformada, resposta impossível de parsear |
| `66` | Recurso não encontrado (EX_NOINPUT) | Biblioteca ou ID de doc não encontrado (API 404) |
| `69` | Serviço indisponível após retry (EX_UNAVAILABLE) | API fora do ar após todas as tentativas esgotadas |
| `74` | Erro de I/O ou rede (EX_IOERR) | Conexão recusada, falha DNS, erro de escrita em disco |
| `77` | Permissão/autenticação negada (EX_NOPERM) | Chave de API inválida (401), rate limited (429) |
| `130` | Interrompido por Ctrl+C (SIGINT) | Usuário pressionou Ctrl+C durante operação |

### Resolução de Problemas

`command not found: context7`
Adicione `~/.cargo/bin` ao seu `PATH`: `export PATH="$HOME/.cargo/bin:$PATH"`

"Nenhuma chave de API configurada"
Execute `context7 keys add ctx7sk-SUA-CHAVE` ou defina a variável `CONTEXT7_API_KEYS`.

"Biblioteca não encontrada" (404)
O ID da biblioteca deve vir de `context7 library NOME`. Copie o caminho `/org/repo` dos resultados da busca, não do GitHub.

Rate limit (429)
A CLI faz retry automaticamente com backoff exponencial (até 5 tentativas). Adicione uma segunda chave com `context7 keys add` para aumentar o throughput.

macOS Gatekeeper bloqueia o binário
Execute: `xattr -d com.apple.quarantine context7 && chmod +x context7`
Ou instale via `cargo install context7-cli` para contornar o Gatekeeper completamente.

Sem cores no terminal
Use `--no-color` ou `--plain` para saída sem marcação, ou defina `NO_COLOR=1`. Verifique se o terminal suporta escape sequences ANSI.

`--text` e `--json` juntos causam erro
Essas flags são mutuamente exclusivas. Use uma ou outra: `context7 docs /org/repo --text` ou `context7 docs /org/repo --json`.

`keys remove 0` retorna "Índice inválido"
Os índices são 1-based. Execute `context7 keys list` primeiro para ver os índices atuais, depois use `context7 keys remove <N>` com N ≥ 1.

`context7 library` sem argumentos retorna erro
O nome da biblioteca é obrigatório. Forneça um termo de busca: `context7 library react`

### Novidades na v0.5.0

- Saída NDJSON estruturada — `--json` agora emite um objeto JSON por linha com campos `type` e `timestamp` para processamento em pipelines
- Flags globais — `--no-color`, `--plain`, `-v/--verbose`, `--quiet` para controle completo do comportamento de saída
- Códigos de saída expandidos — códigos compatíveis com sysexits (65, 66, 69, 74, 77, 130) para classificação precisa de erros em scripts
- Zeroize seguro de chaves — chaves de API encapsuladas em newtype seguro com limpeza automática de memória ao sair de escopo
- Builds ARM64 Linux — targets `aarch64-unknown-linux-gnu` e `aarch64-unknown-linux-musl` para Raspberry Pi 4+, AWS Graviton e containers Alpine em ARM
- Controle de cores — variáveis de ambiente `NO_COLOR` e `CLICOLOR_FORCE` suportadas (convenção no-color.org)
- Ctrl+C gracioso — handler de sinal executa limpeza adequada e encerra com código 130

Veja [CHANGELOG.md](CHANGELOG.md) para o histórico completo.

### Documentação Completa

- [HOW_TO_USE.md](docs/HOW_TO_USE.md) — guia passo a passo para Linux, macOS e Windows
- [CROSS_PLATFORM.md](docs/CROSS_PLATFORM.md) — Windows UTF-8, Gatekeeper macOS, Linux musl/Flatpak/Snap, NixOS, GNU Guix
- [context7-skill.md](docs/context7-skill.md) — padrões de integração com LLM
- [context7-llm-rules.md](docs/context7-llm-rules.md) — regras de system prompt para assistentes de IA


## License

Dual-licensed under MIT OR Apache-2.0, at your choice.

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)

Copyright 2026 Danilo Aguiar &lt;daniloaguiarbr@proton.me&gt;
