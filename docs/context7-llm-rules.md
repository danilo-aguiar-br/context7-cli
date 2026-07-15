# context7-cli LLM Rules and Prompt Templates / Regras e Templates de Prompt

---

## English

### Rules for LLM agents using context7-cli

These rules prevent common mistakes when an LLM agent controls the `context7` CLI. Follow them in every session.

#### Rule 1 — Always discover library IDs with `library` first

```bash
# CORRECT: discover the ID before fetching docs
context7 library react --json
# → use the returned "id" field, e.g., "/reactjs/react.dev"
context7 docs /reactjs/react.dev --query "useEffect"

# WRONG: guessing the ID
context7 docs /react/react     # likely 404
context7 docs react            # will fail
```

#### Rule 2 — Always use `--json` for machine-readable output

```bash
# CORRECT
context7 library react --json | jaq '.[0].id'

# AVOID for scripting (ANSI colors break parsing)
context7 library react
```

#### Rule 3 — Use `--text` when inserting docs into LLM context

```bash
# CORRECT: plain text, no ANSI, clean for context windows
context7 docs /reactjs/react.dev --query "hooks" --text

# AVOID: JSON is verbose; use only when you need the snippet structure
context7 docs /reactjs/react.dev --query "hooks" --json
```

#### Rule 4 — Never expose full API keys in prompts or logs

```bash
# CORRECT: list shows masked values
context7 keys list
# Output: 1. ctx7sk-abcd...xyz9

# AVOID: export shows full values — only use in secure environments
context7 keys export
```

#### Rule 5 — Respect trust scores

- Trust score range: 0–10
- Score ≥ 8: high confidence — use directly
- Score 5–7: medium confidence — verify with the user before using
- Score < 5: low confidence — flag it, prefer another source

#### Rule 6 — Handle errors without crashing the workflow

| Error message | Cause | Agent action |
|---------------|-------|-------------|
| `No API key found` | No key configured | Inform the user; ask them to run `context7 keys add ctx7sk-...` |
| `401 Unauthorized` | Invalid key | Ask user to check and refresh key at context7.com |
| `429 Too Many Requests` | Rate limit (after up to 5 attempts) | Wait 10–30 seconds; suggest adding more keys |
| `Network error` | No internet or timeout | Report to user; do not loop indefinitely |
| `400 Bad Request` | Malformed query | Sanitize the query and retry once |

#### Rule 7 — Mutually exclusive flags

`--text` and `--json` cannot be combined. The CLI will return an error if both are passed.

```bash
# WRONG
context7 docs /reactjs/react.dev --text --json

# CORRECT
context7 docs /reactjs/react.dev --text
# or
context7 docs /reactjs/react.dev --json
```

#### Rule 8 — Use `--json` for structured NDJSON output

When parsing output programmatically, always use `--json`. Output is Newline-Delimited JSON (NDJSON) — each line is a self-contained JSON object with `type` and `timestamp` fields:

```bash
context7 library react --json
# {"type":"library","timestamp":"2026-04-16T12:00:00Z","data":{...}}
```

#### Rule 9 — Use `--plain` or `--no-color` to avoid ANSI escape sequences

When the output will be processed by another tool or inserted into LLM context, use `--plain` or `--no-color` to strip ANSI escape sequences:

```bash
# CORRECT: no escape sequences in output
context7 --plain library react
context7 --no-color docs /reactjs/react.dev --query "hooks" --text

# ALSO CORRECT: via environment variable
NO_COLOR=1 context7 library react
```

#### Rule 10 — Use `-v` for debug when output is unexpected

When the CLI returns unexpected output or silently fails, use `-v` (verbose) to get debug-level logging on stderr:

```bash
context7 -v library react --json 2>debug.log
```

#### Rule 11 — Interpret BSD exit codes precisely

| Exit code | Meaning | Agent action |
|-----------|---------|-------------|
| 0 | Success | Parse output normally |
| 2 | Invalid CLI arguments | Fix the command syntax and retry |
| 65 | Input data error | Sanitize the query string and retry once |
| 66 | Library not found | The library ID is wrong — run `context7 library` to discover the correct ID |
| 69 | Service unavailable | API is down — wait 30 seconds and retry once |
| 77 | Invalid or missing API keys | Keys need reconfiguration — ask the user to run `context7 keys add ctx7sk-...` |
| 130 | Interrupted by user (Ctrl+C) | User cancelled — do NOT retry automatically |

```bash
# Example: handle exit code 66 in a script
context7 docs /invalid/id --query "test" --json
if [ $? -eq 66 ]; then
  echo "Library not found — re-discovering..."
  context7 library "original-name" --json
fi
```

#### Rule 12 — Language selection

Use `--lang` or `CONTEXT7_LANG` to control the UI language in multilingual pipelines:

```bash
# Force English for consistent parsing in scripts
context7 --lang en library react --json

# Set permanently for a session
export CONTEXT7_LANG=en

# Portuguese output for Brazilian users
context7 --lang pt docs /reactjs/react.dev --query "hooks" --text
```

Auto-detect order: `--lang` flag → `CONTEXT7_LANG` env var → system locale (any `pt*` locale → Portuguese) → English default.

---

### Prompt templates

#### Template 1 — Library discovery

Use this when the user mentions a library by name and you need to find its Context7 ID:

```
I need to find documentation for the library "<LIBRARY_NAME>".

Step 1: run this command and parse the JSON output:
```bash
context7 library <LIBRARY_NAME> --json
```

From the response array, select the entry with the highest `trustScore`.
Present the top 3 results to the user in this format:
- `<id>`: <title> (trust score: <score>)

Ask the user: "Which of these libraries would you like documentation for?"

Once they confirm, proceed to Template 2 with the selected `id`.
```

#### Template 2 — Documentation fetch

Use this after you have the library ID from Template 1:

```
I will now fetch the documentation for `<LIBRARY_ID>` about: <TOPIC>.

Run:
```bash
context7 docs <LIBRARY_ID> --query "<TOPIC>" --text
```

Insert the entire output into the conversation context.
If the output is longer than 4000 tokens, summarize the most relevant sections.
Always cite the `codeId` field (source URL) provided in the output.
```

#### Template 3 — API key management guidance

Use when the user needs to set up or troubleshoot API keys:

```
To configure context7-cli API keys, run these commands:

1. Add your API key (get one at https://context7.com):
```bash
context7 keys add ctx7sk-YOUR-KEY-HERE
```

2. Verify it was stored:
```bash
context7 keys list
```

3. (Optional) Add more keys for better rate limit handling:
```bash
context7 keys add ctx7sk-SECOND-KEY
context7 keys add ctx7sk-THIRD-KEY
```

4. Check where the config file is stored:
```bash
context7 keys path
```

5. (Optional) Check current key status:
```bash
context7 keys list
# Rotation is automatic — the CLI shuffles keys per request
```
```

#### Template 4 — Multi-library comparison

Use when the user wants to compare documentation across multiple libraries:

```
I will fetch documentation for multiple libraries to compare them.

For each library in the list, run:
```bash
context7 docs <library-id> --query "<TOPIC>" --text
```

Present the results side by side with headers:
## <Library Name> (<library-id>)
<documentation content>

After presenting all results, summarize the key differences.
```

#### Template 5 — Pipeline: search + fetch + summarize

Use for automated documentation research sessions:

```
Documentation research session for topic: "<TOPIC>"

Step 1 — Find relevant libraries:
```bash
context7 library <TOPIC> --json
```
Select the top 3 results by trustScore.

Step 2 — Fetch docs for each:
```bash
context7 docs <id1> --query "<TOPIC>" --text
context7 docs <id2> --query "<TOPIC>" --text
context7 docs <id3> --query "<TOPIC>" --text
```

Step 3 — Synthesize: provide a single summary that covers:
- What each library does
- How they handle <TOPIC>
- Key differences and recommendations
```

---

### Integration with shell scripts

**Bash script: search and save documentation**

```bash
#!/usr/bin/env bash
set -euo pipefail

LIBRARY="${1:?Usage: $0 <library-name> [query]}"
QUERY="${2:-}"
OUTPUT_DIR="${3:-/tmp/context7-docs}"

mkdir -p "$OUTPUT_DIR"

echo "Searching for library: $LIBRARY"
ID=$(context7 library "$LIBRARY" --json | jaq -r '.[0].id')

if [ -z "$ID" ]; then
  echo "Error: library '$LIBRARY' not found" >&2
  exit 1
fi

echo "Found: $ID"
echo "Fetching documentation..."

if [ -n "$QUERY" ]; then
  context7 docs "$ID" --query "$QUERY" --text > "$OUTPUT_DIR/${ID##/}.md"
else
  context7 docs "$ID" --text > "$OUTPUT_DIR/${ID##/}.md"
fi

echo "Saved to: $OUTPUT_DIR/${ID##/}.md"
```

**PowerShell script: key rotation check**

```powershell
$keys = context7 keys list 2>&1
if ($LASTEXITCODE -ne 0 -or $keys -match "No keys") {
    Write-Warning "No API keys configured. Run: context7 keys add ctx7sk-..."
    exit 1
}
Write-Host "Keys OK: $keys"
```

---

## Português

### Regras para agentes LLM usando context7-cli

Estas regras previnem erros comuns quando um agente LLM controla a CLI `context7`. Siga-as em todas as sessões.

#### Regra 1 — Sempre descubra IDs de bibliotecas com `library` primeiro

```bash
# CORRETO: descobrir o ID antes de buscar a documentação
context7 library react --json
# → usar o campo "id" retornado, ex: "/reactjs/react.dev"
context7 docs /reactjs/react.dev --query "useEffect"

# ERRADO: adivinhar o ID
context7 docs /react/react     # provavelmente 404
context7 docs react            # vai falhar
```

#### Regra 2 — Sempre use `--json` para saída legível por máquina

```bash
# CORRETO
context7 library react --json | jaq '.[0].id'

# EVITAR em scripting (cores ANSI quebram o parsing)
context7 library react
```

#### Regra 3 — Use `--text` ao inserir documentação no contexto de LLM

```bash
# CORRETO: texto plano, sem ANSI, limpo para janelas de contexto
context7 docs /reactjs/react.dev --query "hooks" --text

# EVITAR: JSON é verboso; use apenas quando precisar da estrutura de snippets
context7 docs /reactjs/react.dev --query "hooks" --json
```

#### Regra 4 — Nunca exponha chaves de API completas em prompts ou logs

```bash
# CORRETO: list mostra valores mascarados
context7 keys list
# Saída: 1. ctx7sk-abcd...xyz9

# EVITAR: export mostra valores completos — use apenas em ambientes seguros
context7 keys export
```

#### Regra 5 — Respeite os trust scores

- Faixa de trust score: 0–10
- Score ≥ 8: alta confiança — use diretamente
- Score 5–7: confiança média — verifique com o usuário antes de usar
- Score < 5: baixa confiança — sinalize, prefira outra fonte

#### Regra 6 — Trate erros sem quebrar o workflow

| Mensagem de erro | Causa | Ação do agente |
|------------------|-------|----------------|
| `Nenhuma chave de API encontrada` | Nenhuma chave configurada | Informar o usuário; pedir para executar `context7 keys add ctx7sk-...` |
| `401 Unauthorized` | Chave inválida | Pedir ao usuário que verifique e renove a chave em context7.com |
| `429 Too Many Requests` | Rate limit (após até 5 tentativas) | Aguardar 10–30 segundos; sugerir adicionar mais chaves |
| Erro de rede | Sem internet ou timeout | Reportar ao usuário; não fazer loop indefinido |
| `400 Bad Request` | Query malformada | Sanitizar a query e tentar novamente uma vez |

#### Regra 7 — Flags mutuamente exclusivas

`--text` e `--json` não podem ser combinadas. A CLI retornará erro se ambas forem passadas.

```bash
# ERRADO
context7 docs /reactjs/react.dev --text --json

# CORRETO
context7 docs /reactjs/react.dev --text
# ou
context7 docs /reactjs/react.dev --json
```

#### Regra 8 — Use `--json` para saida NDJSON estruturada

Ao fazer parsing de saida programaticamente, sempre use `--json`. A saida e Newline-Delimited JSON (NDJSON) — cada linha e um objeto JSON autonomo com campos `type` e `timestamp`:

```bash
context7 library react --json
# {"type":"library","timestamp":"2026-04-16T12:00:00Z","data":{...}}
```

#### Regra 9 — Use `--plain` ou `--no-color` para evitar escape sequences ANSI

Quando a saida sera processada por outra ferramenta ou inserida em contexto de LLM, use `--plain` ou `--no-color` para remover escape sequences ANSI:

```bash
# CORRETO: sem escape sequences na saida
context7 --plain library react
context7 --no-color docs /reactjs/react.dev --query "hooks" --text

# TAMBEM CORRETO: via variavel de ambiente
NO_COLOR=1 context7 library react
```

#### Regra 10 — Use `-v` para debug quando a saida for inesperada

Quando a CLI retornar saida inesperada ou falhar silenciosamente, use `-v` (verbose) para obter logging nivel debug no stderr:

```bash
context7 -v library react --json 2>debug.log
```

#### Regra 11 — Interprete codigos de saida BSD com precisao

| Codigo de saida | Significado | Acao do agente |
|-----------------|-------------|----------------|
| 0 | Sucesso | Fazer parsing normalmente |
| 2 | Argumentos de CLI invalidos | Corrigir a sintaxe do comando e tentar novamente |
| 65 | Erro nos dados de entrada | Sanitizar a string de query e tentar uma vez |
| 66 | Biblioteca nao encontrada | O ID da biblioteca esta errado — executar `context7 library` para descobrir o ID correto |
| 69 | Servico indisponivel | API fora do ar — aguardar 30 segundos e tentar uma vez |
| 77 | Chaves de API invalidas ou ausentes | Chaves precisam de reconfiguracao — pedir ao usuario para executar `context7 keys add ctx7sk-...` |
| 130 | Interrompido pelo usuario (Ctrl+C) | O usuario cancelou — NAO tentar novamente automaticamente |

```bash
# Exemplo: tratar codigo de saida 66 em um script
context7 docs /id/invalido --query "test" --json
if [ $? -eq 66 ]; then
  echo "Biblioteca nao encontrada — redescobrindo..."
  context7 library "nome-original" --json
fi
```

#### Regra 12 — Seleção de idioma

Use `--lang` ou `CONTEXT7_LANG` para controlar o idioma da interface em pipelines multilíngues:

```bash
# Forçar inglês para parsing consistente em scripts
context7 --lang en library react --json

# Configurar permanentemente para uma sessão
export CONTEXT7_LANG=pt

# Saída em português para usuários brasileiros
context7 --lang pt docs /reactjs/react.dev --query "hooks" --text
```

Ordem de detecção: flag `--lang` → variável `CONTEXT7_LANG` → locale do sistema (qualquer locale `pt*` → português) → inglês (padrão).

---

### Templates de prompt

#### Template 1 — Descoberta de biblioteca

Use quando o usuário mencionar uma biblioteca pelo nome e você precisar encontrar o ID no Context7:

```
Preciso encontrar documentação para a biblioteca "<NOME_DA_BIBLIOTECA>".

Passo 1: execute este comando e faça o parsing do output JSON:
```bash
context7 library <NOME_DA_BIBLIOTECA> --json
```

Do array de resposta, selecione a entrada com o maior `trustScore`.
Apresente os 3 primeiros resultados ao usuário neste formato:
- `<id>`: <title> (trust score: <score>)

Pergunte ao usuário: "Sobre qual dessas bibliotecas você quer documentação?"

Após confirmar, prossiga para o Template 2 com o `id` selecionado.
```

#### Template 2 — Busca de documentação

Use após obter o ID da biblioteca pelo Template 1:

```
Vou agora buscar a documentação de `<LIBRARY_ID>` sobre: <TÓPICO>.

Execute:
```bash
context7 docs <LIBRARY_ID> --query "<TÓPICO>" --text
```

Insira o output completo no contexto da conversa.
Se o output tiver mais de 4000 tokens, resuma as seções mais relevantes.
Sempre cite o campo `codeId` (URL da fonte) fornecido no output.
```

#### Template 3 — Orientação para gerenciamento de chaves de API

Use quando o usuário precisar configurar ou solucionar problemas com chaves:

```
Para configurar as chaves de API do context7-cli, execute estes comandos:

1. Adicionar sua chave de API (obtenha uma em https://context7.com):
```bash
context7 keys add ctx7sk-SUA-CHAVE-AQUI
```

2. Verificar se foi salva:
```bash
context7 keys list
```

3. (Opcional) Adicionar mais chaves para melhor tolerância a rate limit:
```bash
context7 keys add ctx7sk-SEGUNDA-CHAVE
context7 keys add ctx7sk-TERCEIRA-CHAVE
```

4. Ver onde o arquivo de config está salvo:
```bash
context7 keys path
```

5. (Opcional) Verificar o status das chaves:
```bash
context7 keys list
# A rotação é automática — a CLI embaralha chaves por requisição
```
```

#### Template 4 — Comparação de múltiplas bibliotecas

Use quando o usuário quiser comparar documentação entre várias bibliotecas:

```
Vou buscar documentação de múltiplas bibliotecas para compará-las.

Para cada biblioteca na lista, execute:
```bash
context7 docs <id-da-biblioteca> --query "<TÓPICO>" --text
```

Apresente os resultados lado a lado com cabeçalhos:
## <Nome da Biblioteca> (<id-da-biblioteca>)
<conteúdo da documentação>

Após apresentar todos os resultados, resuma as principais diferenças.
```

#### Template 5 — Pipeline: buscar + buscar docs + resumir

Use para sessões automatizadas de pesquisa de documentação:

```
Sessão de pesquisa de documentação sobre: "<TÓPICO>"

Passo 1 — Encontrar bibliotecas relevantes:
```bash
context7 library <TÓPICO> --json
```
Selecione os 3 primeiros resultados por trustScore.

Passo 2 — Buscar docs de cada uma:
```bash
context7 docs <id1> --query "<TÓPICO>" --text
context7 docs <id2> --query "<TÓPICO>" --text
context7 docs <id3> --query "<TÓPICO>" --text
```

Passo 3 — Sintetizar: forneça um resumo único que cubra:
- O que cada biblioteca faz
- Como cada uma trata <TÓPICO>
- Principais diferenças e recomendações
```

---

### Integração com scripts shell

**Script Bash: buscar e salvar documentação**

```bash
#!/usr/bin/env bash
set -euo pipefail

BIBLIOTECA="${1:?Uso: $0 <nome-da-biblioteca> [query]}"
QUERY="${2:-}"
DIR_SAIDA="${3:-/tmp/context7-docs}"

mkdir -p "$DIR_SAIDA"

echo "Buscando biblioteca: $BIBLIOTECA"
ID=$(context7 library "$BIBLIOTECA" --json | jaq -r '.[0].id')

if [ -z "$ID" ]; then
  echo "Erro: biblioteca '$BIBLIOTECA' não encontrada" >&2
  exit 1
fi

echo "Encontrada: $ID"
echo "Buscando documentação..."

if [ -n "$QUERY" ]; then
  context7 docs "$ID" --query "$QUERY" --text > "$DIR_SAIDA/${ID##/}.md"
else
  context7 docs "$ID" --text > "$DIR_SAIDA/${ID##/}.md"
fi

echo "Salvo em: $DIR_SAIDA/${ID##/}.md"
```

**Script PowerShell: verificação de rotação de chaves**

```powershell
$chaves = context7 keys list 2>&1
if ($LASTEXITCODE -ne 0 -or $chaves -match "Nenhuma chave") {
    Write-Warning "Nenhuma chave de API configurada. Execute: context7 keys add ctx7sk-..."
    exit 1
}
Write-Host "Chaves OK: $chaves"
```
