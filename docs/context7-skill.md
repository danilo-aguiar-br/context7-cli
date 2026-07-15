---
name: context7-skill
description: Use the context7-cli binary to search library documentation and API references from the terminal via the Context7 REST API. Invoke when the user asks about library APIs, wants up-to-date documentation, or mentions Context7. Covers library search, documentation fetch, key management, and output formatting.
version: 0.5.0
---

## English

### When to invoke this skill

- User asks about a library's API, method signatures, or configuration options
- User wants up-to-date documentation for any library or framework
- User mentions: Context7, context7-cli, library search, API docs, documentation fetch
- Before writing code that depends on an external library's API
- When training data may be outdated for a specific library version

### Prerequisites

```bash
# 1. Verify context7 is installed
which context7           # Unix/Linux/macOS
where context7           # Windows CMD
Get-Command context7     # Windows PowerShell

# 2. Verify API keys are configured
context7 keys list

# 3. If no keys are listed, add one (ask the user for their key or direct them to https://context7.com)
context7 keys add ctx7sk-THEIR-KEY-HERE
```

If `context7` is not installed:

```bash
cargo install context7-cli    # requires Rust toolchain from https://rustup.rs
```

### Usage patterns

#### Pattern 1 — Search for a library by name

```bash
context7 library <name> [optional-context] --json
```

- Always pass `--json` for machine-readable output
- `optional-context` improves result ranking (e.g., `"effect hooks"`, `"async channels"`)

Examples:

```bash
context7 library react --json
context7 library axum "middleware routing" --json
context7 library tokio "mpsc channel" --json
context7 library vue --json
```

#### Pattern 2 — Fetch documentation for a specific library

```bash
context7 docs <library-id> --query "<question>" --text
# Short form: -q is an alias for --query
context7 docs <library-id> -q "<question>" --text
```

- Use `library-id` from Pattern 1 output (format: `/org/repo`)
- Use `--text` for LLM context insertion (plain text, no ANSI)
- Use `--json` for structured parsing
- Use `-q` as a short alias for `--query`

Examples:

```bash
context7 docs /reactjs/react.dev --query "useEffect and cleanup" --text
context7 docs /tokio-rs/tokio -q "spawn_blocking use cases" --text
context7 docs /rust-lang/rust -q "lifetime annotations" --text
context7 docs /tokio-rs/axum --query "tower middleware" --json
```

#### Pattern 3 — 2-step discovery workflow (recommended)

Always use `library` before `docs` to get the exact library ID:

```bash
# Step 1: Find the library ID
context7 library react --json
# Outputs: [{"id": "/reactjs/react.dev", "title": "React", "trustScore": 9.8}, ...]

# Step 2: Fetch the docs using that ID
context7 docs /reactjs/react.dev --query "useState and useEffect" --text
```

### Output parsing

**Library search output (`--json`):**

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

- `id`: exact string to pass to `context7 docs`
- `trustScore`: 0–10; results with score < 7 may be less relevant
- Use `jaq '.[0].id'` to extract the top result's ID

**Documentation output (`--json`):**

```json
{
  "id": "/reactjs/react.dev",
  "snippets": [
    {
      "pageTitle": "useEffect",
      "codeTitle": "Basic useEffect example",
      "codeDescription": "The useEffect hook lets you synchronize a component with an external system.",
      "codeLanguage": "javascript",
      "codeTokens": 42,
      "codeId": "https://react.dev/reference/react/useEffect",
      "codeList": [
        { "language": "javascript", "code": "useEffect(() => { /* cleanup */ }, [deps]);" }
      ],
      "relevance": 0.95,
      "model": "gemini-2.5-flash"
    }
  ]
}
```

- `snippets[].codeTitle`: title of the code snippet
- `snippets[].codeId`: source URL for citation

### Examples with expected outputs

**Example 1 — Find React library**

```bash
context7 library react --json
```

Expected:

```json
[
  {"id": "/reactjs/react.dev", "title": "React", "trustScore": 9.8},
  {"id": "/preactjs/preact", "title": "Preact", "trustScore": 8.1}
]
```

**Example 2 — Fetch useEffect docs**

```bash
context7 docs /reactjs/react.dev --query "useEffect cleanup function" --text
```

Expected: plain Markdown text about `useEffect`, its cleanup function, and dependency array — ready to insert into an LLM context window.

**Example 3 — Key management check**

```bash
context7 keys list
```

Expected:

```
1. ctx7sk-abcd...xyz9 (added on 2026-04-08)
```

(Values are always masked — never full keys in output.)

### Error handling

| Exit code | Meaning | Action |
|-----------|---------|--------|
| 0 | Success | Parse output |
| 1 | General runtime error | Show error message to user |
| 2 | Invalid CLI arguments | Fix command syntax |
| 65 | Input data error | Sanitize query and retry |
| 66 | Library not found or invalid ID | Verify ID via `context7 library` |
| 69 | Service unavailable | Wait and retry |
| 74 | I/O error (config file) | Check file permissions |
| 77 | Invalid or missing API keys | Run `context7 keys add ctx7sk-...` |
| 130 | Interrupted by Ctrl+C | Do not retry |

Common error messages and their solutions:

- `No API key found` → run `context7 keys add ctx7sk-...`
- `401 Unauthorized` → key invalid; run `context7 keys remove <N>` and add a new one
- `429 Too Many Requests` → already retried with backoff; wait ~10s and retry
- `All API keys failed` → all keys exhausted; get new key from context7.com

### Key management commands

```bash
context7 keys add <key>        # add a key (persisted to XDG config)
context7 keys list             # list all keys (masked)
context7 keys remove <index>   # remove key by 1-based index
context7 keys clear --yes      # remove all keys (no prompt)
context7 keys path             # show config file path
context7 keys export           # export to .env format (full values)
context7 keys import <file>    # import from .env file
# Rotation is automatic — each request shuffles keys randomly.
```

### Global flags (v0.5.0+)

| Flag | Description |
|------|-------------|
| `--no-color` | Disable all ANSI colors and decorations |
| `--plain` | Alias for `--no-color` |
| `--verbose`, `-v` | Enable debug-level logging to stderr |
| `--quiet` | Suppress all non-essential output |

```bash
# Clean output for LLM pipelines
context7 --plain docs /reactjs/react.dev --query "hooks" --text

# Debug a failing request
context7 -v library react --json

# Script-friendly: only data on stdout
context7 --quiet library react --json | jaq '.[0].id'
```

### NDJSON output format (v0.5.0+)

When `--json` is used, output is Newline-Delimited JSON (NDJSON) — each line is a self-contained JSON object with `type` and `timestamp`:

```bash
context7 library react --json
# {"type":"library","timestamp":"2026-04-16T12:00:00Z","data":{...}}
```

### Exit codes (BSD sysexits, v0.5.0+)

| Exit code | Name | Meaning |
|-----------|------|---------|
| 0 | `EX_OK` | Success |
| 1 | `EX_GENERAL` | General runtime error |
| 2 | `EX_USAGE` | Invalid CLI arguments |
| 65 | `EX_DATAERR` | Input data error |
| 66 | `EX_NOINPUT` | Library not found or invalid ID |
| 69 | `EX_UNAVAILABLE` | Service unavailable (API down) |
| 74 | `EX_IOERR` | I/O error (config file failure) |
| 77 | `EX_NOPERM` | Invalid or missing API keys |
| 130 | `EX_SIGINT` | Interrupted by Ctrl+C |

### NO_COLOR environment variable (v0.5.0+)

Set `NO_COLOR` (any value) to disable all ANSI colors — equivalent to `--no-color`:

```bash
NO_COLOR=1 context7 library react --json
```

### Language control (v0.2.0+)

```bash
# Force English output (useful in multilingual environments)
context7 --lang en library react --json

# Force Portuguese output
context7 --lang pt docs /reactjs/react.dev --query "hooks" --text

# Permanent override via env var
export CONTEXT7_LANG=en
```

Auto-detect order: `--lang` flag → `CONTEXT7_LANG` env var → system locale → English default.

### Environment variables

| Variable | Purpose |
|----------|---------|
| `CONTEXT7_API_KEYS` | Comma-separated API keys (overrides config file) |
| `CONTEXT7_LANG` | UI language: `en` or `pt` |
| `CONTEXT7_HOME` | Alternative XDG config directory (mainly for tests and CI) |
| `NO_COLOR` | When set (any value), disables all ANSI colors |
| `CLICOLOR_FORCE` | When set to `1`, forces colors even when stdout is not a TTY |
| `RUST_LOG` | Log level: `error`, `warn`, `info`, `debug`, `trace` |

### Rules for this skill

1. **Always use `library` before `docs`** — never guess a library ID
2. **Always pass `--json`** for machine-readable output
3. **Never expose full API keys** — use `context7 keys list` (masked) or avoid showing key operations in output
4. **Respect trust scores** — flag results with `trustScore < 7` as lower confidence
5. **Use `--text` for LLM context** — cleaner than `--json` when inserting docs into a prompt
6. **Handle errors gracefully** — the CLI retries automatically; if it still fails, show the error message to the user
7. **Use `--lang en`** for consistent output language in multilingual pipelines

---

## Português

### Quando invocar esta skill

- O usuário pergunta sobre a API de uma biblioteca, assinaturas de métodos ou opções de configuração
- O usuário quer documentação atualizada de qualquer biblioteca ou framework
- O usuário menciona: Context7, context7-cli, busca de biblioteca, API docs, busca de documentação
- Antes de escrever código que depende da API de uma biblioteca externa
- Quando os dados de treinamento podem estar desatualizados para uma versão específica de uma biblioteca

### Pré-requisitos

```bash
# 1. Verificar se o context7 está instalado
which context7           # Unix/Linux/macOS
where context7           # Windows CMD
Get-Command context7     # Windows PowerShell

# 2. Verificar se as chaves de API estão configuradas
context7 keys list

# 3. Se nenhuma chave estiver listada, adicionar uma (peça a chave ao usuário ou direcione para https://context7.com)
context7 keys add ctx7sk-CHAVE-DO-USUARIO
```

Se o `context7` não estiver instalado:

```bash
cargo install context7-cli    # requer Rust toolchain de https://rustup.rs
```

### Padrões de uso

#### Padrão 1 — Buscar uma biblioteca por nome

```bash
context7 library <nome> [contexto-opcional] --json
```

- Sempre passe `--json` para saída legível por máquina
- `contexto-opcional` melhora o ranking dos resultados (ex: `"hooks de efeito"`, `"canais async"`)

Exemplos:

```bash
context7 library react --json
context7 library axum "middleware rotas" --json
context7 library tokio "canal mpsc" --json
context7 library vue --json
```

#### Padrão 2 — Buscar documentação de uma biblioteca específica

```bash
context7 docs <id-da-biblioteca> --query "<pergunta>" --text
# Forma curta: -q é um alias para --query
context7 docs <id-da-biblioteca> -q "<pergunta>" --text
```

- Use o `id-da-biblioteca` do output do Padrão 1 (formato: `/org/repo`)
- Use `--text` para inserir no contexto de um LLM (texto plano, sem ANSI)
- Use `--json` para parsing estruturado
- Use `-q` como alias curto para `--query`

Exemplos:

```bash
context7 docs /reactjs/react.dev --query "useEffect e cleanup" --text
context7 docs /tokio-rs/tokio -q "casos de uso do spawn_blocking" --text
context7 docs /rust-lang/rust -q "anotações de lifetime" --text
context7 docs /tokio-rs/axum --query "tower middleware" --json
```

#### Padrão 3 — Fluxo de descoberta em 2 passos (recomendado)

Sempre use `library` antes de `docs` para obter o ID exato da biblioteca:

```bash
# Passo 1: Encontrar o ID da biblioteca
context7 library react --json
# Saída: [{"id": "/reactjs/react.dev", "title": "React", "trustScore": 9.8}, ...]

# Passo 2: Buscar a documentação com esse ID
context7 docs /reactjs/react.dev --query "useState e useEffect" --text
```

### Parsing do output

**Output de busca de biblioteca (`--json`):**

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

- `id`: string exata para passar para `context7 docs`
- `trustScore`: 0–10; resultados com score < 7 podem ser menos relevantes
- Use `jaq '.[0].id'` para extrair o ID do primeiro resultado

**Output de documentação (`--json`):**

```json
{
  "id": "/reactjs/react.dev",
  "snippets": [
    {
      "pageTitle": "useEffect",
      "codeTitle": "Exemplo básico de useEffect",
      "codeDescription": "O hook useEffect permite sincronizar um componente com um sistema externo.",
      "codeLanguage": "javascript",
      "codeTokens": 42,
      "codeId": "https://react.dev/reference/react/useEffect",
      "codeList": [
        { "language": "javascript", "code": "useEffect(() => { /* cleanup */ }, [deps]);" }
      ],
      "relevance": 0.95,
      "model": "gemini-2.5-flash"
    }
  ]
}
```

- `snippets[].codeTitle`: título do snippet de código
- `snippets[].codeId`: URL da fonte para citação

### Exemplos com outputs esperados

**Exemplo 1 — Encontrar a biblioteca React**

```bash
context7 library react --json
```

Esperado:

```json
[
  {"id": "/reactjs/react.dev", "title": "React", "trustScore": 9.8},
  {"id": "/preactjs/preact", "title": "Preact", "trustScore": 8.1}
]
```

**Exemplo 2 — Buscar documentação do useEffect**

```bash
context7 docs /reactjs/react.dev --query "função de cleanup do useEffect" --text
```

Esperado: texto Markdown plano sobre `useEffect`, sua função de cleanup e array de dependências — pronto para inserir na janela de contexto de um LLM.

**Exemplo 3 — Verificação de gerenciamento de chaves**

```bash
context7 keys list
```

Esperado:

```
1. ctx7sk-abcd...xyz9 (adicionada em 2026-04-08)
```

(Os valores são sempre mascarados — nunca chaves completas na saída.)

### Tratamento de erros

| Código de saída | Significado | Ação |
|-----------------|-------------|------|
| 0 | Sucesso | Fazer parsing do output |
| 1 | Erro geral de runtime | Mostrar mensagem de erro ao usuário |
| 2 | Argumentos de CLI inválidos | Corrigir sintaxe do comando |
| 65 | Erro nos dados de entrada | Sanitizar query e tentar novamente |
| 66 | Biblioteca não encontrada ou ID inválido | Verificar ID via `context7 library` |
| 69 | Serviço indisponível | Aguardar e tentar novamente |
| 74 | Erro de I/O (arquivo de config) | Verificar permissões do arquivo |
| 77 | Chaves de API inválidas ou ausentes | Executar `context7 keys add ctx7sk-...` |
| 130 | Interrompido por Ctrl+C | Não tentar novamente |

Mensagens de erro comuns e soluções:

- `Nenhuma chave de API encontrada` → execute `context7 keys add ctx7sk-...`
- `401 Unauthorized` → chave inválida; execute `context7 keys remove <N>` e adicione uma nova
- `429 Too Many Requests` → já fez retry com backoff; aguarde ~10s e tente novamente
- `Todas as chaves de API falharam` → todas as chaves esgotadas; obtenha nova chave em context7.com

### Comandos de gerenciamento de chaves

```bash
context7 keys add <chave>       # adicionar chave (persistida no config XDG)
context7 keys list              # listar todas as chaves (mascaradas)
context7 keys remove <índice>   # remover chave por índice 1-based
context7 keys clear --yes       # remover todas as chaves (sem prompt)
context7 keys path              # exibir caminho do arquivo config
context7 keys export            # exportar em formato .env (valores completos)
context7 keys import <arquivo>  # importar de arquivo .env
# A rotação é automática — cada requisição embaralha as chaves aleatoriamente.
```

### Flags globais (v0.5.0+)

| Flag | Descricao |
|------|-----------|
| `--no-color` | Desabilita todas as cores e decoracoes ANSI |
| `--plain` | Alias para `--no-color` |
| `--verbose`, `-v` | Habilita logging nivel debug no stderr |
| `--quiet` | Suprime toda saida nao-essencial |

```bash
# Saida limpa para pipelines de LLM
context7 --plain docs /reactjs/react.dev --query "hooks" --text

# Depurar uma requisicao com falha
context7 -v library react --json

# Para scripts: apenas dados no stdout
context7 --quiet library react --json | jaq '.[0].id'
```

### Formato de saida NDJSON (v0.5.0+)

Quando `--json` e usado, a saida e Newline-Delimited JSON (NDJSON) — cada linha e um objeto JSON autonomo com `type` e `timestamp`:

```bash
context7 library react --json
# {"type":"library","timestamp":"2026-04-16T12:00:00Z","data":{...}}
```

### Codigos de saida (BSD sysexits, v0.5.0+)

| Codigo de saida | Nome | Significado |
|-----------------|------|-------------|
| 0 | `EX_OK` | Sucesso |
| 1 | `EX_GENERAL` | Erro geral de runtime |
| 2 | `EX_USAGE` | Argumentos de CLI invalidos |
| 65 | `EX_DATAERR` | Erro nos dados de entrada |
| 66 | `EX_NOINPUT` | Biblioteca nao encontrada ou ID invalido |
| 69 | `EX_UNAVAILABLE` | Servico indisponivel (API fora do ar) |
| 74 | `EX_IOERR` | Erro de I/O (falha no arquivo de config) |
| 77 | `EX_NOPERM` | Chaves de API invalidas ou ausentes |
| 130 | `EX_SIGINT` | Interrompido por Ctrl+C |

### Variavel de ambiente NO_COLOR (v0.5.0+)

Defina `NO_COLOR` (qualquer valor) para desabilitar todas as cores ANSI — equivalente a `--no-color`:

```bash
NO_COLOR=1 context7 library react --json
```

### Controle de idioma (v0.2.0+)

```bash
# Forçar saída em inglês (útil em ambientes multilíngues)
context7 --lang en library react --json

# Forçar saída em português
context7 --lang pt docs /reactjs/react.dev --query "hooks" --text

# Override permanente via variável de ambiente
export CONTEXT7_LANG=pt
```

Ordem de detecção: flag `--lang` → variável `CONTEXT7_LANG` → locale do sistema → inglês (padrão).

### Variáveis de ambiente

| Variável | Finalidade |
|----------|-----------|
| `CONTEXT7_API_KEYS` | Chaves de API separadas por vírgula (sobrepõe o arquivo de config) |
| `CONTEXT7_LANG` | Idioma da interface: `en` ou `pt` |
| `CONTEXT7_HOME` | Diretório XDG alternativo (principalmente para testes e CI) |
| `NO_COLOR` | Quando definida (qualquer valor), desabilita todas as cores ANSI |
| `CLICOLOR_FORCE` | Quando definida como `1`, força cores mesmo quando stdout não é TTY |
| `RUST_LOG` | Nível de log: `error`, `warn`, `info`, `debug`, `trace` |

### Regras desta skill

1. **Sempre use `library` antes de `docs`** — nunca adivinhe um ID de biblioteca
2. **Sempre passe `--json`** para saída legível por máquina
3. **Nunca exponha chaves de API completas** — use `context7 keys list` (mascarado) ou evite mostrar operações de chaves no output
4. **Respeite os trust scores** — sinalize resultados com `trustScore < 7` como de menor confiança
5. **Use `--text` para contexto de LLM** — mais limpo que `--json` ao inserir docs em um prompt
6. **Trate erros com elegância** — a CLI faz retry automaticamente; se ainda falhar, mostre a mensagem de erro ao usuário
7. **Use `--lang en`** para linguagem de output consistente em pipelines multilíngues
