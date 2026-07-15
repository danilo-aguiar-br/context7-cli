# Cross-Platform Guide — context7-cli

**EN:** This guide covers platform-specific behavior, known quirks, and installation notes for context7-cli on Windows, macOS, and Linux distributions.

**PT:** Este guia cobre o comportamento específico por plataforma, particularidades conhecidas e instruções de instalação do context7-cli no Windows, macOS e distribuições Linux.

---

## Windows — UTF-8 Console Setup

**EN:** Since v0.2.4, context7-cli automatically configures the Windows console to UTF-8 (code page 65001) at startup. This ensures that accented characters, special symbols, and bilingual output (EN/PT) render correctly in `cmd.exe`, PowerShell, and Windows Terminal.

**PT:** Desde a v0.2.4, o context7-cli configura automaticamente o console do Windows para UTF-8 (code page 65001) na inicialização. Isso garante que caracteres acentuados, símbolos especiais e a saída bilíngue (EN/PT) sejam exibidos corretamente no `cmd.exe`, PowerShell e Windows Terminal.

### How it works / Como funciona

```rust
// src/main.rs — chamada no startup do main() antes de qualquer I/O
#[cfg(windows)]
configurar_console_utf8();
```

Internally / Internamente:
- `SetConsoleOutputCP(65001)` — sets output code page to UTF-8
- `SetConsoleCP(65001)` — sets input code page to UTF-8
- `colored::control::set_virtual_terminal(true)` — enables ANSI escape sequences for colored output

**EN:** If `set_virtual_terminal` fails (very old `cmd.exe` without Virtual Terminal Processing), colors are automatically disabled to prevent raw escape sequences from appearing.

**PT:** Se `set_virtual_terminal` falhar (cmd.exe muito antigo sem Virtual Terminal Processing), as cores são desabilitadas automaticamente para evitar que escape sequences brutas sejam exibidas.

### Configuration path / Caminho de configuração

```
%APPDATA%\context7\config\config.toml
```

Example / Exemplo:

```
C:\Users\YourName\AppData\Roaming\context7\config\config.toml
```

### Windows reserved filename validation (v0.5.0+)

**EN:** `CONTEXT7_HOME` values are validated against Windows reserved filenames (`CON`, `PRN`, `NUL`, `AUX`, `COM1`..`COM9`, `LPT1`..`LPT9`). Any path containing a reserved name as a component is rejected to prevent path injection attacks.

**PT:** Valores de `CONTEXT7_HOME` são validados contra nomes de arquivo reservados do Windows (`CON`, `PRN`, `NUL`, `AUX`, `COM1`..`COM9`, `LPT1`..`LPT9`). Qualquer caminho contendo um nome reservado como componente é rejeitado para prevenir ataques de injeção de caminho.

### Override via `CONTEXT7_HOME`

**EN:** Set `CONTEXT7_HOME` to use a custom directory. Path traversal (`..`) is rejected for security. Windows reserved filenames are also rejected.

**PT:** Defina `CONTEXT7_HOME` para usar um diretório personalizado. Path traversal (`..`) é rejeitado por segurança. Nomes de arquivo reservados do Windows também são rejeitados.

```powershell
$env:CONTEXT7_HOME = "C:\MyConfig"
context7 keys list
```

---

## Windows — cmd.exe vs PowerShell vs Windows Terminal

**EN:** While `context7-cli` automatically configures UTF-8 via `SetConsoleOutputCP(65001)`, there are behavioral differences between Windows shells that affect environment variables, pipe encoding, and line endings.

**PT:** Embora o `context7-cli` configure automaticamente UTF-8 via `SetConsoleOutputCP(65001)`, existem diferenças comportamentais entre os shells do Windows que afetam variáveis de ambiente, encoding de pipes e terminações de linha.

### Environment variables / Variáveis de ambiente

| Shell | Syntax | Example |
|-------|--------|---------|
| **cmd.exe** | `set VAR=value` | `set CONTEXT7_LANG=pt` (session-scoped) |
| **PowerShell** | `$env:VAR = "value"` | `$env:CONTEXT7_LANG = "pt"` (session-scoped) |
| **PowerShell (permanent)** | `[Environment]::SetEnvironmentVariable("VAR","value","User")` | Persists across sessions |
| **Windows Terminal** | Inherits from the underlying shell | Same as cmd.exe or PowerShell |

### Pipe encoding / Encoding em pipes

**EN:** Pipes in `cmd.exe` use the console code page (65001 after context7 sets it). PowerShell pipes use UTF-16LE internally but convert to UTF-8 when writing to external processes. In practice, `context7 library react --json | jaq '.[0].id'` works identically in both.

**PT:** Pipes no `cmd.exe` usam o code page do console (65001 após o context7 configurar). Pipes do PowerShell usam UTF-16LE internamente mas convertem para UTF-8 ao escrever para processos externos. Na prática, `context7 library react --json | jaq '.[0].id'` funciona de forma idêntica em ambos.

### Line endings / Terminações de linha

**EN:** When importing `.env` files created on Windows (`\r\n` line endings), `context7 keys import` handles both `\r\n` and `\n` transparently. No manual conversion needed.

**PT:** Ao importar arquivos `.env` criados no Windows (terminações de linha `\r\n`), `context7 keys import` lida com `\r\n` e `\n` de forma transparente. Nenhuma conversão manual necessária.

### Recommended shell / Shell recomendado

**EN:** Windows Terminal + PowerShell 7+ provides the best experience: full ANSI color support, proper UTF-8 rendering, and native pipe compatibility.

**PT:** Windows Terminal + PowerShell 7+ oferece a melhor experiência: suporte completo a cores ANSI, renderização UTF-8 correta e compatibilidade nativa com pipes.

---

## macOS — Gatekeeper and Code Signing

### Adhoc signing vs Developer ID / Assinatura adhoc vs Developer ID

**EN:** The GitHub Actions release workflow produces a Universal Binary (Intel x86_64 + Apple Silicon arm64, combined via `lipo`). The signing behavior depends on whether Apple Developer secrets are configured in the repository:

**PT:** O workflow de release do GitHub Actions produz um Universal Binary (Intel x86_64 + Apple Silicon arm64, combinados via `lipo`). O comportamento de assinatura depende de os secrets do Apple Developer estarem configurados no repositório:

| Condition / Condição | Signing / Assinatura | Notarization / Notarização |
|---|---|---|
| `APPLE_TEAM_ID` secret set | Developer ID (full) | Apple notarytool ✓ |
| `APPLE_TEAM_ID` empty/absent | Adhoc (`codesign --sign -`) | Not notarized |

### First-run Gatekeeper warning / Aviso Gatekeeper na primeira execução

**EN:** If the binary is downloaded from GitHub Releases (not notarized), macOS Gatekeeper may block execution with the message _"context7 cannot be opened because it is from an unidentified developer"_. To allow execution:

**PT:** Se o binário for baixado dos GitHub Releases (sem notarização), o macOS Gatekeeper pode bloquear a execução com a mensagem _"context7 não pode ser aberto porque é de um desenvolvedor não identificado"_. Para permitir a execução:

```bash
# Remove the quarantine attribute set by Gatekeeper
# Remove o atributo de quarentena definido pelo Gatekeeper
xattr -d com.apple.quarantine context7

# Make executable if needed / Tornar executável se necessário
chmod +x context7

# Verify the binary is a valid Universal Binary
# Verificar que o binário é um Universal Binary válido
lipo -info context7
# Expected output: Architectures in the fat file: context7 are: x86_64 arm64
```

**EN:** Alternatively, right-click the binary in Finder → Open → Open (bypasses Gatekeeper for that file).

**PT:** Alternativamente, clique com o botão direito no binário no Finder → Abrir → Abrir (ignora o Gatekeeper para esse arquivo).

### Install via cargo / Instalação via cargo

**EN:** The recommended installation method avoids Gatekeeper entirely:

**PT:** O método de instalação recomendado evita o Gatekeeper completamente:

```bash
cargo install context7-cli
```

**EN:** Binaries compiled locally are not subject to Gatekeeper quarantine.

**PT:** Binários compilados localmente não estão sujeitos à quarentena do Gatekeeper.

### Configuration path / Caminho de configuração

```
~/Library/Application Support/context7/config.toml
```

### Override via `CONTEXT7_HOME`

```bash
export CONTEXT7_HOME="$HOME/.myconfig"
context7 keys list
```

---

## Linux — Distribution Guide

### Standard install via cargo / Instalação padrão via cargo

**EN:** Works on any Linux distribution with a glibc ≥ 2.31 (standard since Ubuntu 20.04+, Debian 11+, Fedora 32+, Arch, etc.):

**PT:** Funciona em qualquer distribuição Linux com glibc ≥ 2.31 (padrão desde Ubuntu 20.04+, Debian 11+, Fedora 32+, Arch, etc.):

```bash
cargo install context7-cli
```

**EN:** The binary is installed to `~/.cargo/bin/context7`. Add `~/.cargo/bin` to your `PATH` if not already present.

**PT:** O binário é instalado em `~/.cargo/bin/context7`. Adicione `~/.cargo/bin` ao seu `PATH` se ainda não estiver presente.

### Configuration path (XDG) / Caminho de configuração (XDG)

```
~/.config/context7/config.toml
```

**EN:** Follows the XDG Base Directory Specification via the `directories` crate. Respects `$XDG_CONFIG_HOME` if set.

**PT:** Segue a XDG Base Directory Specification via o crate `directories`. Respeita `$XDG_CONFIG_HOME` se definido.

### Alpine Linux / musl

**EN:** The standard glibc binary does **not** run on Alpine Linux (which uses musl libc). Use the static musl binary from the GitHub Releases page:

**PT:** O binário glibc padrão **não** funciona no Alpine Linux (que usa musl libc). Use o binário musl estático da página de GitHub Releases:

```bash
# Download the musl static binary / Baixar o binário musl estático
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-x86_64-unknown-linux-musl.tar.gz \
  | tar xz

# Make executable and move to PATH / Tornar executável e mover para PATH
chmod +x context7
mv context7 /usr/local/bin/
```

**EN:** The musl binary is fully static — no shared library dependencies.

**PT:** O binário musl é completamente estático — sem dependências de bibliotecas compartilhadas.

### NixOS

**EN:** A `flake.nix` is not included in the repository. The recommended approach is to enter a shell with the Rust toolchain from nixpkgs and use `cargo install`:

**PT:** Um `flake.nix` não está incluído no repositório. A abordagem recomendada é entrar em um shell com a toolchain Rust do nixpkgs e usar `cargo install`:

```bash
# Temporary shell with Rust toolchain / Shell temporário com toolchain Rust
nix-shell -p rustc cargo

# Install inside the shell / Instalar dentro do shell
cargo install context7-cli
```

**EN:** Alternatively, create a `shell.nix` for a reproducible environment:

**PT:** Alternativamente, crie um `shell.nix` para um ambiente reproduzível:

```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = [ pkgs.rustc pkgs.cargo pkgs.pkg-config pkgs.openssl ];
}
```

**EN:** context7-cli uses `rustls` for TLS (pure Rust) — no OpenSSL dependency at runtime. The `openssl` entry above is only needed during compilation on some nixpkgs configurations. If you encounter a link error about OpenSSL, remove it from `buildInputs` and try again.

**PT:** O context7-cli usa `rustls` para TLS (Rust puro) — sem dependência de OpenSSL em runtime. A entrada `openssl` acima é necessária apenas durante a compilação em algumas configurações do nixpkgs. Se encontrar um erro de link sobre OpenSSL, remova-a de `buildInputs` e tente novamente.

**EN:** XDG paths work normally on NixOS. The config file is stored at:

**PT:** Caminhos XDG funcionam normalmente no NixOS. O arquivo de configuração é armazenado em:

```
~/.config/context7/config.toml
```

### GNU Guix

**EN:** Install the Rust toolchain via Guix and use `cargo install`:

**PT:** Instale a toolchain Rust via Guix e use `cargo install`:

```bash
# Install Rust toolchain via Guix / Instalar toolchain Rust via Guix
guix install rust cargo

# Install context7-cli / Instalar context7-cli
cargo install context7-cli
```

**EN:** As with NixOS, context7-cli uses `rustls` for TLS — no OpenSSL runtime dependency. XDG paths (`~/.config/context7/config.toml`) work as expected under Guix.

**PT:** Assim como no NixOS, o context7-cli usa `rustls` para TLS — sem dependência de OpenSSL em runtime. Caminhos XDG (`~/.config/context7/config.toml`) funcionam como esperado no Guix.

### Flatpak

**EN:** The GitHub Actions release workflow builds a Flatpak bundle artifact. Install from a local `.flatpak` file:

**PT:** O workflow de release do GitHub Actions gera um artefato Flatpak bundle. Instalar a partir de um arquivo `.flatpak` local:

```bash
# Add Flathub remote (if not already added) / Adicionar remote Flathub (se não adicionado)
flatpak remote-add --user --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Install the Flatpak bundle / Instalar o bundle Flatpak
flatpak install --user context7-cli.flatpak

# Run / Executar
flatpak run br.com.daniloaguiar.context7-cli library react
```

**EN:** The Flatpak runs in a sandbox with `--share=network` only. It does **not** have `--filesystem=home`, which means the Flatpak sandbox cannot read or write `~/.config/context7/config.toml` from the host. API keys added via `flatpak run` are stored inside the Flatpak sandbox data directory and are **isolated** from keys stored by a `cargo install` binary on the same machine.

**PT:** O Flatpak roda em sandbox com apenas `--share=network`. Ele **não** possui `--filesystem=home`, o que significa que o sandbox do Flatpak não pode ler ou gravar `~/.config/context7/config.toml` do host. Chaves de API adicionadas via `flatpak run` são armazenadas dentro do diretório de dados do sandbox do Flatpak e ficam **isoladas** das chaves armazenadas por um binário `cargo install` na mesma máquina.

**Build details / Detalhes do build:**

| Field / Campo | Value / Valor |
|---|---|
| `app-id` | `br.com.daniloaguiar.context7-cli` |
| `runtime` | `org.freedesktop.Platform` |
| `runtime-version` | `24.08` |
| `sdk-extension` | `org.freedesktop.Sdk.Extension.rust-stable` |
| `finish-args` | `--share=network` |

### Snap

**EN:** The GitHub Actions release workflow builds a Snap artifact. Install from a local `.snap` file:

**PT:** O workflow de release do GitHub Actions gera um artefato Snap. Instalar a partir de um arquivo `.snap` local:

```bash
# Install from local file (unsigned) / Instalar a partir de arquivo local (sem assinatura)
sudo snap install context7-cli_*.snap --dangerous

# Set up alias / Configurar alias
sudo snap alias context7-cli.context7 context7

# Run / Executar
context7 library react
```

**EN:** If published to the Snap Store in the future:

**PT:** Se publicado no Snap Store no futuro:

```bash
sudo snap install context7-cli
```

**Snap configuration details / Detalhes de configuração do Snap:**

| Field / Campo | Value / Valor |
|---|---|
| `base` | `core24` |
| `confinement` | `strict` |
| `grade` | `stable` |
| `plugs` | `network`, `home` |
| `architectures` | `amd64`, `arm64` |

### SELinux / AppArmor

**EN:** No special policy is required. context7-cli is a user-space CLI that:
- Makes outbound HTTPS connections to `api.context7.com`
- Reads/writes configuration in `~/.config/context7/` (or `$XDG_CONFIG_HOME`)
- Has no elevated privileges, no setuid/setgid, no kernel interfaces

**PT:** Nenhuma política especial é necessária. O context7-cli é uma CLI user-space que:
- Faz conexões HTTPS de saída para `api.context7.com`
- Lê/escreve configuração em `~/.config/context7/` (ou `$XDG_CONFIG_HOME`)
- Não possui privilégios elevados, sem setuid/setgid, sem interfaces com o kernel

**EN:** Standard SELinux `unconfined_u:unconfined_r:unconfined_t` context is sufficient. AppArmor does not restrict user binaries by default on Ubuntu.

**PT:** O contexto SELinux padrão `unconfined_u:unconfined_r:unconfined_t` é suficiente. O AppArmor não restringe binários de usuário por padrão no Ubuntu.

---

## Installation Method Comparison / Comparação dos Métodos de Instalação

**EN:** The table below compares the three main Linux installation methods.

**PT:** A tabela abaixo compara os três principais métodos de instalação no Linux.

| | `cargo install` | Flatpak | Snap |
|---|---|---|---|
| **Key isolation / Isolamento de chaves** | None — shared `~/.config/context7/` | Full sandbox — keys isolated from host | Partial — `home` plug, separate data dir |
| **Config path** | `~/.config/context7/config.toml` | Flatpak sandbox data dir | Snap data dir |
| **Updates / Atualizações** | Manual: `cargo install context7-cli --force` | Manual: `flatpak update` | Automatic via Snap Store (when published) |
| **Network access** | Full | `--share=network` only | `network` plug |
| **Installation source** | crates.io | Local `.flatpak` file (GitHub Release) | Local `.snap` file or Snap Store |
| **Architecture** | Native (matches host) | x86_64 (CI artifact) | amd64, arm64 |
| **Root required / Root necessário** | No | No (`--user` flag) | Yes for `snap install` |

**EN:** For most users, `cargo install` provides the simplest experience with no isolation concerns. Use Flatpak or Snap only when sandbox isolation is a requirement.

**PT:** Para a maioria dos usuários, `cargo install` oferece a experiência mais simples sem preocupações de isolamento. Use Flatpak ou Snap apenas quando o isolamento de sandbox for um requisito.

---

## Docker / Containers

**EN:** The static musl binary is ideal for containers — zero shared-library dependencies, minimal image size.

**PT:** O binário musl estático é ideal para containers — zero dependências de bibliotecas compartilhadas, tamanho de imagem mínimo.

### Multi-stage Dockerfile

**EN:** Build from source using a multi-stage image:

**PT:** Compilar a partir do código-fonte usando imagem multi-stage:

```dockerfile
# Build stage / Estágio de compilação
FROM rust:1.75-alpine AS builder
RUN apk add --no-cache musl-dev
RUN cargo install context7-cli

# Runtime stage / Estágio de runtime
FROM alpine:3.20
COPY --from=builder /usr/local/cargo/bin/context7 /usr/local/bin/
ENV CONTEXT7_HOME=/data/context7
ENTRYPOINT ["context7"]
```

### Pre-built binary (smaller build) / Binário pré-compilado (build menor)

**EN:** Use the pre-built musl binary from GitHub Releases for faster Docker builds:

**PT:** Use o binário musl pré-compilado dos GitHub Releases para builds Docker mais rápidos:

```dockerfile
FROM alpine:3.20
ADD https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-x86_64-unknown-linux-musl.tar.gz /tmp/
RUN tar xzf /tmp/context7-x86_64-unknown-linux-musl.tar.gz -C /usr/local/bin/ \
    && rm /tmp/context7-x86_64-unknown-linux-musl.tar.gz
ENV CONTEXT7_HOME=/data/context7
ENTRYPOINT ["context7"]
```

### Key persistence / Persistência de chaves

**EN:** Mount a volume to persist API keys across container restarts:

**PT:** Monte um volume para persistir as chaves de API entre reinicializações do container:

```bash
# Add a key (persisted in the named volume) / Adicionar chave (persistida no volume nomeado)
docker run -v context7-keys:/data/context7 context7-cli keys add ctx7sk-...

# Use the key in subsequent runs / Usar a chave em execuções seguintes
docker run -v context7-keys:/data/context7 context7-cli library react
```

### Environment variables in containers / Variáveis de ambiente em containers

**EN:** Pass keys directly via environment variable — no config file needed:

**PT:** Passe chaves diretamente via variável de ambiente — sem arquivo de configuração necessário:

```bash
docker run -e CONTEXT7_API_KEYS=ctx7sk-key1,ctx7sk-key2 context7-cli library react
```

**EN:** This is the recommended approach for CI/CD pipelines and ephemeral containers.

**PT:** Esta é a abordagem recomendada para pipelines de CI/CD e containers efêmeros.

### Notes / Observações

**EN:**
- The `tar` command in the pre-built Dockerfile example requires Alpine's `tar` package (`apk add tar`) or use `busybox tar` which is included by default.
- The `CONTEXT7_HOME` env var overrides the XDG path inside the container. Set it to a volume-mounted path for key persistence.
- For ARM64 containers (Apple Silicon, AWS Graviton), use the `aarch64-unknown-linux-musl` release artifact instead.

**PT:**
- O comando `tar` no exemplo do Dockerfile pré-compilado requer o pacote `tar` do Alpine (`apk add tar`) ou use o `busybox tar` que é incluído por padrão.
- A variável `CONTEXT7_HOME` sobrescreve o caminho XDG dentro do container. Defina-a para um caminho montado em volume para persistência de chaves.
- Para containers ARM64 (Apple Silicon, AWS Graviton), use o artefato de release `aarch64-unknown-linux-musl` em vez do x86_64.

---

## Summary Table / Tabela Resumo

| Platform / Plataforma | Config Path | UTF-8 Auto | Notes / Notas |
|---|---|---|---|
| Windows (cmd/PS) | `%APPDATA%\context7\config\config.toml` | Yes (v0.2.4+) | ANSI auto-detected |
| macOS | `~/Library/Application Support/context7/config.toml` | N/A | `xattr` if Gatekeeper blocks |
| Linux (glibc) | `~/.config/context7/config.toml` | N/A | XDG compliant |
| Alpine (musl) | `~/.config/context7/config.toml` | N/A | Use musl static binary |
| NixOS | `~/.config/context7/config.toml` | N/A | `nix-shell -p rustc cargo` |
| GNU Guix | `~/.config/context7/config.toml` | N/A | `guix install rust cargo` |
| Linux Flatpak | Sandbox data dir (isolated) | N/A | `--share=network` only, keys isolated |
| Linux Snap | Snap data dir | N/A | `strict` confinement, `home` plug |

**EN:** All platforms respect the `CONTEXT7_HOME` environment variable as a universal config path override. Path traversal (`..`) is always rejected.

**PT:** Todas as plataformas respeitam a variável de ambiente `CONTEXT7_HOME` como um override universal do caminho de configuração. Path traversal (`..`) é sempre rejeitado.

---

## CI/CD Integration / Integração com CI/CD

**EN:** context7-cli works seamlessly in CI/CD pipelines. Set `CONTEXT7_API_KEYS` as a secret and install via `cargo install`.

**PT:** O context7-cli funciona perfeitamente em pipelines CI/CD. Configure `CONTEXT7_API_KEYS` como segredo e instale via `cargo install`.

### GitHub Actions

```yaml
name: Docs validation
on: [push]
jobs:
  check-docs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Install context7-cli
        run: cargo install context7-cli
      - name: Verify library docs available
        env:
          CONTEXT7_API_KEYS: ${{ secrets.CONTEXT7_API_KEY }}
        run: |
          context7 library react --json | jaq '.[0].id'
          context7 docs /reactjs/react.dev --query "useEffect" --text > /dev/null
          echo "✅ Context7 API accessible"
```

### GitLab CI

```yaml
docs-check:
  image: rust:latest
  script:
    - cargo install context7-cli
    - context7 library react --json
  variables:
    CONTEXT7_API_KEYS: $CONTEXT7_SECRET
```

### Environment variables for CI / Variáveis de ambiente para CI

| Variable | Purpose | Required |
|----------|---------|----------|
| `CONTEXT7_API_KEYS` | API key(s), comma-separated | Yes |
| `CONTEXT7_LANG` | Force output language (`en` / `pt`) | No (defaults to `en`) |
| `RUST_LOG` | Debug logging level | No |

---

## ARM64 Linux Support (v0.5.0+) / Suporte ARM64 Linux (v0.5.0+)

**EN:** Since v0.5.0, the release pipeline produces pre-built binaries for ARM64 Linux:

**PT:** Desde a v0.5.0, o pipeline de release produz binários pré-compilados para ARM64 Linux:

| Target | Use case / Caso de uso |
|--------|------------------------|
| `aarch64-unknown-linux-gnu` | Raspberry Pi OS, Ubuntu on ARM, AWS Graviton, Ampere |
| `aarch64-unknown-linux-musl` | Alpine Linux on ARM, minimal ARM containers |

```bash
# ARM64 glibc binary / Binário ARM64 glibc
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-aarch64-unknown-linux-gnu.tar.gz \
  | tar xz
chmod +x context7 && mv context7 /usr/local/bin/

# ARM64 musl static binary / Binário ARM64 musl estático
curl -L https://github.com/danilo-aguiar-br/context7-cli/releases/latest/download/context7-aarch64-unknown-linux-musl.tar.gz \
  | tar xz
chmod +x context7 && mv context7 /usr/local/bin/
```

### mimalloc allocator for musl targets

**EN:** The musl binaries (`x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`) use the `mimalloc` allocator instead of the default musl allocator. This provides significantly better performance for allocation-heavy workloads such as JSON serialization and HTTP response parsing.

**PT:** Os binários musl (`x86_64-unknown-linux-musl` e `aarch64-unknown-linux-musl`) usam o alocador `mimalloc` em vez do alocador padrão do musl. Isso proporciona performance significativamente melhor para workloads intensivos em alocação, como serialização JSON e parsing de respostas HTTP.

---

## Color Control and ASCII Fallback (v0.5.0+) / Controle de Cores e Fallback ASCII (v0.5.0+)

### NO_COLOR / CLICOLOR_FORCE

**EN:** `context7-cli` respects the [NO_COLOR](https://no-color.org/) standard:

**PT:** A `context7-cli` respeita o padrão [NO_COLOR](https://no-color.org/):

| Variable / Variável | Effect / Efeito |
|----------------------|-----------------|
| `NO_COLOR` | When set (any value), disables all ANSI colors / Quando definida (qualquer valor), desabilita todas as cores ANSI |
| `CLICOLOR_FORCE` | When set to `1`, forces colors even when stdout is not a TTY / Quando definida como `1`, força cores mesmo quando stdout não é TTY |

```bash
# Disable colors / Desabilitar cores
NO_COLOR=1 context7 library react

# Force colors in a pipe / Forçar cores em pipe
CLICOLOR_FORCE=1 context7 library react | less -R
```

### ASCII fallback

**EN:** When stdout is not a TTY (piped or redirected) or `NO_COLOR` is set, Unicode decorations are replaced with ASCII equivalents:

**PT:** Quando stdout não é TTY (redirecionado ou em pipe) ou `NO_COLOR` está definida, decorações Unicode são substituídas por equivalentes ASCII:

| Unicode | ASCII | Context / Contexto |
|---------|-------|---------------------|
| `\u25b8` | `>` | Bullet points / Marcadores |
| `\u2500` | `-` | Horizontal rules / Linhas horizontais |

**EN:** This ensures clean output in terminals without Unicode support and in CI/CD logs.

**PT:** Isso garante saída limpa em terminais sem suporte a Unicode e em logs de CI/CD.

---

## Signal Handling (v0.5.0+) / Tratamento de Sinais (v0.5.0+)

**EN:** `context7-cli` handles interruption signals gracefully:

- Ctrl+C sends SIGINT on Unix and CTRL_C_EVENT on Windows
- The CLI catches the signal, cancels in-flight HTTP requests, and exits with code 130
- No partial output is written to stdout after interruption
- Exit code 130 follows the Unix convention of `128 + signal number` (SIGINT = 2)

**PT:** A `context7-cli` trata sinais de interrupção de forma graceful:

- Ctrl+C envia SIGINT no Unix e CTRL_C_EVENT no Windows
- A CLI captura o sinal, cancela requisições HTTP em andamento e encerra com código 130
- Nenhuma saída parcial é escrita no stdout após a interrupção
- O código de saída 130 segue a convenção Unix de `128 + número do sinal` (SIGINT = 2)

---

## Known Platform Gaps / Gaps Conhecidos de Plataforma

### macOS Gatekeeper / Notarization

**EN:** Pre-built binaries from GitHub Releases use adhoc signing by default. If Apple Gatekeeper blocks execution:

```bash
# Option 1: Remove quarantine attribute
xattr -d com.apple.quarantine context7 && chmod +x context7

# Option 2: Verify binary (if signed with Developer ID)
spctl --assess --type execute context7

# Option 3: Install via cargo (bypasses Gatekeeper entirely)
cargo install context7-cli
```

Automated notarization is available in the release pipeline when `APPLE_TEAM_ID` and `APPLE_CERTIFICATE` secrets are configured.

**PT:** Binários pré-compilados do GitHub Releases usam assinatura adhoc por padrão. Se o Gatekeeper da Apple bloquear a execução:

```bash
# Opção 1: Remover atributo de quarentena
xattr -d com.apple.quarantine context7 && chmod +x context7

# Opção 2: Verificar binário (se assinado com Developer ID)
spctl --assess --type execute context7

# Opção 3: Instalar via cargo (contorna o Gatekeeper completamente)
cargo install context7-cli
```

Notarização automatizada está disponível no pipeline de release quando os segredos `APPLE_TEAM_ID` e `APPLE_CERTIFICATE` estão configurados.
