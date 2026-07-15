# Changelog / Registro de Mudanças

All notable changes to `context7-cli` will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## English

---

## [0.5.2] — 2026-07-15

### Changed

- **Ownership / repository migration**: crate owner on crates.io is now `danilo-aguiar-br` (previous publisher account became `ghost_405118`).
- **Canonical GitHub repository** moved to `https://github.com/danilo-aguiar-br/context7-cli` (`repository` / `homepage` in `Cargo.toml`).
- **Removed all GitHub Actions CI/CD** (`.github/workflows/`, dependency automation). Releases are published manually; no Actions recreated on the new repo.
- Contact email aligned to `daniloaguiarbr@proton.me`.
- Packaging snap version aligned to `0.5.2`.

### Notes

- crates.io only updates `repository`/`homepage` when a **new version** is published; this release carries those metadata fixes.
- Uncommitted `0.5.1` work (health subcommand, quiet flag, etc.) is included in this tree; `0.5.1` was already on crates.io from the previous account.

---

## [0.5.1] — 2026-04-16

### Added

- `context7 health [--json]` subcommand: validates config directory, API keys, and reachability of the Context7 API. Returns BSD sysexits (0 ok, 64 config error, 69 API unavailable, 77 permission denied) for automation and CI integration.
- `src/health.rs` module with end-to-end liveness checks reusing existing storage and API client.
- `tests/health_integration.rs`: E2E coverage for the `health` subcommand.
- 3 new CLI integration tests: `testa_quiet_suprime_stdout_em_keys_list_vazio`, `testa_quiet_preserva_stderr_em_erro_sem_chave`, `testa_sem_quiet_produz_stdout_em_keys_list_vazio`.
- 2 inline tests in `src/platform.rs`: `testa_inicializar_plataforma_nao_faz_panic`, `testa_inicializar_plataforma_e_noop_em_nao_windows`.
- 7 new `Mensagem::Health*` variants in `i18n.rs` with exhaustive EN/PT match preserved.
- CI `version-sync` job in `.github/workflows/ci.yml`: compares `Cargo.toml` version against `packaging/snap/snapcraft.yaml` version to prevent packaging drift across releases.

### Fixed

- `--quiet` flag is now actually enforced. Previously declared but unused; stdout suppression is gated through `output::definir_silencioso()` backed by a `OnceLock<bool>` for thread-safe one-time initialization. All 34 `println!` call sites in `output.rs` now route through `imprimir_linha()`, which honors the silent state. `stderr` remains intact so errors stay visible even under `--quiet`.

### Changed

- `packaging/snap/snapcraft.yaml` version bumped from 0.4.3 to 0.5.1 (aligned with crate version via new CI gate).
- `.gitignore` now excludes `AGENTS.md` and `ralph-loop.local.md` from version control.

---

## [0.5.0] — 2026-04-16

### Added

- 4 global CLI flags: `--no-color`, `--plain`, `--verbose`/`-v` (count-based), `--quiet`
- NDJSON output format with `type` and `timestamp` fields when `--json` is active — LLM-native contract
- Signal handling: graceful Ctrl+C shutdown via `tokio::select!` with exit code 130
- BSD exit codes: 65 (EX_DATAERR), 66 (EX_NOINPUT), 69 (EX_UNAVAILABLE), 74 (EX_IOERR), 77 (EX_NOPERM)
- `NO_COLOR` env var support (any value disables colors)
- `CLICOLOR_FORCE=1` env var support (forces colors even in pipes)
- ASCII fallback for Unicode symbols when stdout is not a TTY or `NO_COLOR` is set
- ARM Linux prebuilt binaries: `aarch64-unknown-linux-gnu` and `aarch64-unknown-linux-musl`
- Windows reserved filename validation (CON, PRN, AUX, NUL, COM0-9, LPT0-9) in `CONTEXT7_HOME`
- Unicode NFC path normalization for macOS HFS+ compatibility

### Changed

- `main()` signature changed from `-> anyhow::Result<()>` to `()` for precise exit code control
- Extracted `src/platform.rs` module from `main.rs` (Windows console initialization)
- Replaced `rand 0.8` with `fastrand 2` (lighter dependency, Fisher-Yates shuffle)

### Security

- API keys now wrapped in `ChaveApi` newtype with `#[derive(Zeroize, ZeroizeOnDrop)]` — memory is zeroed on drop
- `ChaveApi` implements masked `Debug`/`Display` — keys never leak to logs
- Added `mimalloc` allocator for musl targets (7x performance improvement over musl malloc)
- Fixed `rustls-webpki` vulnerability (RUSTSEC-2026-0098/0099): updated 0.103.10 → 0.103.12

### Technical

- `Vec::with_capacity()` in 3 hot paths of `storage.rs`
- Added `unicode-normalization` dependency for NFC path normalization
- Added `zeroize` dependency with `derive` feature
- Added `mimalloc` conditional dependency for `cfg(target_env = "musl")`
- Added `signal` feature to `tokio`
- Cargo.toml `[profile.release]`: lto="fat", codegen-units=1, panic="abort", strip="symbols"
- Updated Cargo.toml `exclude` list (+`.claude/`, `AGENTS.md`, `crates.md`, `ralph-loop.local.md`)
- 227 cargo tests passing, zero clippy warnings, zero fmt diffs

---

## [0.4.3] — 2026-04-09

### Docs

- Updated README "What's New" from v0.4.0 to v0.4.3 (EN + PT) — now highlights 70/70 empirical QA, Snap/Flatpak CI fixes, resilient release pipeline, shell completions, and dynamic user-agent.
- Updated `context7-skill.md` frontmatter version from 0.4.0 to 0.4.3.
- Expanded `CROSS_PLATFORM.md` (+128 lines): new sections for Windows cmd.exe vs PowerShell vs Windows Terminal comparison, CI/CD integration examples (GitHub Actions + GitLab CI), and Known Platform Gaps (aarch64-linux, macOS Gatekeeper/notarization).
- Updated `packaging/snap/snapcraft.yaml` version to 0.4.3.

### QA

- **70/70 empirical tests passed**: every command, alias, flag, environment variable, and edge case from HOW_TO_USE.md validated end-to-end on a live system with real API keys and isolated temp directories — zero regressions found.
- 227 cargo tests passing (97 unit + 22 api + 63 cli + 33 i18n + 12 storage), zero warnings.

---

## [0.4.2] — 2026-04-09

### Fixed

- **Snap CI build**: removed invalid `build-attributes` dict format — snapcraft 8.x requires a list of strings, not a list of dicts
- **Flatpak CI build**: removed non-existent `--allow-network` flag from `flatpak-builder` command — network access is already the default behavior

---

## [0.4.1] — 2026-04-09

### Fixed

- **Snap CI build**: pipeline now copies `snapcraft.yaml` from `packaging/snap/` to repo root before running `snapcore/action-build@v1` — fixes "Project file not found" error that prevented snap artifact generation.
- **Flatpak CI build**: removed `--offline` flag and `CARGO_NET_OFFLINE` from flatpak manifest; added `--allow-network` to `flatpak-builder` in CI — fixes build failure caused by missing `vendor/` directory.
- **Release pipeline resilience**: `publish-github-release` job now uses `always()` condition requiring only `validate`, `build-matrix`, and `macos-universal` to succeed — Snap/Flatpak failures no longer block binary releases for all platforms.
- **Snap version sync**: `packaging/snap/snapcraft.yaml` updated from `0.2.9` to `0.4.1`.

### Added

- **SECURITY.md**: security policy with GitHub Security Advisories as primary channel and email as fallback (EN + PT).
- **CODE_OF_CONDUCT.md**: Contributor Covenant v2.1 reference (EN + PT).

---

## [0.4.0] — 2026-04-09

### Added

- **Shell completions** for bash, zsh, fish, PowerShell, and elvish via `context7 completions <SHELL>` (alias: `completion`). Generates shell-specific completion scripts that can be installed with a single redirect. Tab-complete all subcommands, flags, and options without memorizing them.
- **"Why context7-cli?" section** in README (EN + PT) — highlights zero context switching, pipe-friendly output, SSH compatibility, and CI/CD readiness.

### Fixed

- **User-agent now uses `env!("CARGO_PKG_VERSION")`** instead of a hardcoded version string. HTTP requests across all releases now report the exact version from `Cargo.toml` — no more stale `context7-cli/0.2.4` headers in newer releases.

### Docs

- Updated "What's New" section from v0.2.9 to v0.4.0 in README (EN + PT).
- Added shell completion installation instructions to README (Quick Reference + dedicated section) and HOW_TO_USE.md (Step 10 / Passo 10 bilíngue).
- Created `CONTRIBUTING.md` with dev setup, test workflow, code style, commit conventions, and PR checklist (EN + PT).

### Technical

- **64/64 empirical tests passed**: every command in HOW_TO_USE.md validated end-to-end on a live system with real API keys — zero regressions found.
- All canonical decisions preserved.

---

## [0.5.2] — 2026-07-15 — Português

### Alterado

- **Migração de ownership / repositório**: owner do crate no crates.io passa a ser `danilo-aguiar-br` (conta anterior virou `ghost_405118`).
- **Repositório canônico no GitHub** movido para `https://github.com/danilo-aguiar-br/context7-cli` (`repository` / `homepage` no `Cargo.toml`).
- **Removido todo CI/CD de GitHub Actions** (`.github/workflows/`, dependency automation). Releases são publicadas manualmente; Actions **não** são recriadas no repo novo.
- E-mail de contato alinhado a `daniloaguiarbr@proton.me`.
- Versão do packaging snap alinhada a `0.5.2`.

### Notas

- O crates.io só atualiza `repository`/`homepage` ao publicar uma **nova versão**; este release carrega esses metadados.
- O trabalho local de `0.5.1` (health, quiet, etc.) está nesta árvore; `0.5.1` já estava no crates.io pela conta anterior.

---

## [0.5.1] — 2026-04-16 — Português

### Adicionado

- Subcomando `context7 health [--json]`: valida diretório de configuração, chaves de API e alcance da API Context7. Retorna exit codes BSD sysexits (0 ok, 64 erro de config, 69 API indisponível, 77 permissão negada) para integração com automação e CI.
- Módulo `src/health.rs` com checagens end-to-end reutilizando storage e cliente de API existentes.
- `tests/health_integration.rs`: cobertura E2E do subcomando `health`.
- 3 novos testes de integração CLI: `testa_quiet_suprime_stdout_em_keys_list_vazio`, `testa_quiet_preserva_stderr_em_erro_sem_chave`, `testa_sem_quiet_produz_stdout_em_keys_list_vazio`.
- 2 testes inline em `src/platform.rs`: `testa_inicializar_plataforma_nao_faz_panic`, `testa_inicializar_plataforma_e_noop_em_nao_windows`.
- 7 novas variantes `Mensagem::Health*` em `i18n.rs` com match EN/PT exaustivo preservado.
- Job `version-sync` no CI em `.github/workflows/ci.yml`: compara versão do `Cargo.toml` com versão do `packaging/snap/snapcraft.yaml` para prevenir drift de empacotamento entre releases.

### Corrigido

- Flag `--quiet` agora realmente efetiva. Antes era declarada mas não consumida; a supressão de stdout passa pelo gate `output::definir_silencioso()` sustentado por `OnceLock<bool>` com inicialização thread-safe única. Todas as 34 chamadas `println!` em `output.rs` passam por `imprimir_linha()`, que respeita o estado silencioso. `stderr` permanece intacto, garantindo que erros continuem visíveis mesmo sob `--quiet`.

### Alterado

- Versão do `packaging/snap/snapcraft.yaml` atualizada de 0.4.3 para 0.5.1 (alinhada com a versão da crate via novo gate de CI).
- `.gitignore` agora exclui `AGENTS.md` e `ralph-loop.local.md` do controle de versão.

---

## [0.5.0] — 2026-04-16 — Português

### Adicionado

- 4 flags globais de CLI: `--no-color`, `--plain`, `--verbose`/`-v` (contagem), `--quiet`
- Formato NDJSON com campos `type` e `timestamp` quando `--json` ativo — contrato nativo para LLMs
- Signal handling: encerramento graceful via Ctrl+C com `tokio::select!` e exit code 130
- Exit codes BSD: 65 (EX_DATAERR), 66 (EX_NOINPUT), 69 (EX_UNAVAILABLE), 74 (EX_IOERR), 77 (EX_NOPERM)
- Suporte a env var `NO_COLOR` (qualquer valor desabilita cores)
- Suporte a env var `CLICOLOR_FORCE=1` (força cores mesmo em pipes)
- Fallback ASCII para símbolos Unicode quando stdout não é TTY ou `NO_COLOR` está setado
- Binários ARM Linux pré-compilados: `aarch64-unknown-linux-gnu` e `aarch64-unknown-linux-musl`
- Validação de nomes reservados Windows (CON, PRN, AUX, NUL, COM0-9, LPT0-9) em `CONTEXT7_HOME`
- Normalização Unicode NFC para paths — compatibilidade com macOS HFS+

### Alterado

- Assinatura de `main()` mudou de `-> anyhow::Result<()>` para `()` para controle preciso de exit codes
- Extraído módulo `src/platform.rs` de `main.rs` (inicialização de console Windows)
- Substituído `rand 0.8` por `fastrand 2` (dependência mais leve, Fisher-Yates shuffle)

### Segurança

- Chaves API encapsuladas em newtype `ChaveApi` com `#[derive(Zeroize, ZeroizeOnDrop)]` — memória zerada ao drop
- `ChaveApi` implementa `Debug`/`Display` mascarados — chaves nunca vazam em logs
- Adicionado allocator `mimalloc` para targets musl (melhoria de 7x sobre malloc do musl)
- Corrigida vulnerabilidade `rustls-webpki` (RUSTSEC-2026-0098/0099): atualizado 0.103.10 → 0.103.12

### Técnico

- `Vec::with_capacity()` em 3 hot paths de `storage.rs`
- Adicionada dependência `unicode-normalization` para normalização NFC
- Adicionada dependência `zeroize` com feature `derive`
- Adicionada dependência condicional `mimalloc` para `cfg(target_env = "musl")`
- Adicionada feature `signal` ao `tokio`
- Cargo.toml `[profile.release]`: lto="fat", codegen-units=1, panic="abort", strip="symbols"
- Atualizado `exclude` do Cargo.toml (+`.claude/`, `AGENTS.md`, `crates.md`, `ralph-loop.local.md`)
- 227 testes cargo passando, zero warnings clippy, zero diferenças fmt

---

## [0.4.3] — 2026-04-09 — Português

### Documentação

- Atualizado README "Novidades" de v0.4.0 para v0.4.3 (EN + PT) — agora destaca 70/70 QA empírico, correções Snap/Flatpak CI, pipeline resiliente, shell completions e user-agent dinâmico.
- Atualizado frontmatter de versão do `context7-skill.md` de 0.4.0 para 0.4.3.
- Expandido `CROSS_PLATFORM.md` (+128 linhas): novas seções para comparação Windows cmd.exe vs PowerShell vs Windows Terminal, exemplos de integração CI/CD (GitHub Actions + GitLab CI), e Gaps Conhecidos de Plataforma (aarch64-linux, macOS Gatekeeper/notarização).
- Atualizado versão do `packaging/snap/snapcraft.yaml` para 0.4.3.

### QA

- **70/70 testes empíricos aprovados**: cada comando, alias, flag, variável de ambiente e edge case do HOW_TO_USE.md validado end-to-end em sistema real com chaves de API reais e diretórios temporários isolados — zero regressões encontradas.
- 227 testes cargo passando (97 unitários + 22 api + 63 cli + 33 i18n + 12 storage), zero warnings.

---

## [0.4.2] — 2026-04-09 — Português

### Corrigido

- **Build Snap no CI**: removido formato dict inválido em `build-attributes` — snapcraft 8.x exige lista de strings, não lista de dicts
- **Build Flatpak no CI**: removida flag `--allow-network` inexistente no comando `flatpak-builder` — acesso à rede já é o comportamento padrão

---

## [0.4.1] — 2026-04-09 — Português

### Corrigido

- **Build Snap no CI**: pipeline agora copia `snapcraft.yaml` de `packaging/snap/` para a raiz do repositório antes de executar `snapcore/action-build@v1` — corrige erro "Project file not found" que impedia a geração do artefato snap.
- **Build Flatpak no CI**: removida flag `--offline` e `CARGO_NET_OFFLINE` do manifesto flatpak; adicionado `--allow-network` ao `flatpak-builder` no CI — corrige falha de build causada pela ausência do diretório `vendor/`.
- **Resiliência do pipeline de release**: job `publish-github-release` agora usa condição `always()` exigindo apenas sucesso de `validate`, `build-matrix` e `macos-universal` — falhas de Snap/Flatpak não bloqueiam mais releases de binários para todas as plataformas.
- **Sincronização de versão do Snap**: `packaging/snap/snapcraft.yaml` atualizado de `0.2.9` para `0.4.1`.

### Adicionado

- **SECURITY.md**: política de segurança com GitHub Security Advisories como canal primário e email como fallback (EN + PT).
- **CODE_OF_CONDUCT.md**: referência ao Contributor Covenant v2.1 (EN + PT).

---

## [0.4.0] — 2026-04-09 — Português

### Adicionado

- **Autocompletar para bash, zsh, fish, PowerShell e elvish** via `context7 completions <SHELL>` (alias: `completion`). Gera scripts de autocompletar específicos para cada shell instaláveis com um único redirecionamento. Complete por tab todos os subcomandos, flags e opções sem precisar memorizá-los.
- **Seção "Por que o context7-cli?"** no README (EN + PT) — destaca zero troca de contexto, saída amigável para pipes, compatibilidade com SSH e suporte nativo a CI/CD.

### Corrigido

- **User-agent agora usa `env!("CARGO_PKG_VERSION")`** em vez de string hardcoded. Requisições HTTP em todas as versões agora reportam a versão exata do `Cargo.toml` — sem mais headers `context7-cli/0.2.4` em versões mais recentes.

### Documentação

- Seção "Novidades" atualizada de v0.2.9 para v0.4.0 no README (EN + PT).
- Instruções de instalação de autocompletar adicionadas ao README (Referência Rápida + seção dedicada) e HOW_TO_USE.md (Step 10 / Passo 10 bilíngue).
- Criado `CONTRIBUTING.md` com setup de desenvolvimento, workflow de testes, estilo de código, convenções de commit e checklist para PRs (EN + PT).

### Técnico

- **64/64 testes empíricos aprovados**: cada comando do HOW_TO_USE.md validado end-to-end em sistema real com chaves de API — zero regressões encontradas.
- Todas as decisões canônicas preservadas.

---

## [0.3.0] — 2026-04-09

### Fixed

- **`keys list --json` now returns human-readable `added_at`**: was outputting raw RFC3339 with nanoseconds (`2026-04-09T18:02:45.943772067+00:00`), now returns `2026-04-09 18:02:45` — consistent with the text-mode output and documentation.
- **`keys clear` confirmation prompt now respects `--lang`**: the interactive prompt was hardcoded in Portuguese; it now uses the i18n system (`Mensagem::ConfirmarRemoverTodas`) and displays in the user's selected language.

### Changed (Architecture)

- **All `println!` centralized in `output.rs`**: removed 8 direct `println!`/`print!` calls from `storage.rs` (5) and `cli.rs` (3). New functions in `output.rs`: `exibir_json_array_vazio`, `exibir_json_bruto`, `exibir_caminho_config`, `exibir_chave_exportada`, `exibir_json_resultados`, `exibir_texto_plano`, `confirmar_clear`. `output.rs` is now the single module responsible for all terminal I/O.

### Added (Tests)

- **15 new tests** (219 total, 0 failing):
  - 3 JSON validation tests (valid parse, expected fields, human-readable `added_at`)
  - 2 CRLF parsing tests (Windows line endings in `.env` files)
  - 8 i18n tests (EN/PT for `keys add`, `keys remove`, `keys clear --yes`, API key error)
  - 2 cross-platform tests (path traversal rejection, UTF-8 Portuguese output)

### Added (Docs)

- **README.md completely rewritten**: new sections for Key Management, Environment Variables, Integration Patterns, Quick Reference cheat sheet, and Troubleshooting. Removed inflated release notes (moved to CHANGELOG).
- **CROSS_PLATFORM.md expanded**: added NixOS and GNU Guix installation guides, documented Flatpak key isolation, added Flatpak vs Snap vs Cargo comparison table.

### Added (CI)

- **musl binary smoke test**: `ci.yml` now builds and executes the Alpine/musl binary (`--version` + `--help`) instead of just compiling metadata.
- **Universal Binary smoke test**: `release.yml` now runs `--version` + `--help` on the lipo-combined macOS Universal Binary before publishing.

---

## [0.2.9] — 2026-04-09

### Fixed

- **Removed redundant `use serde_json;` import** in `tests/cli_integration.rs` that triggered `clippy::single_component_path_imports` warning with `-D warnings`, causing CI failure on all three platforms (ubuntu, macos, windows) and blocking the release workflow.

### Added (Docs)

- **`keys list --json` documentation** in `docs/HOW_TO_USE.md` (EN + PT) — the `--json` flag for `keys list` was added in v0.2.8 but not documented in the usage guide.

### Technical

- Cleaned 93 residual `.profraw` instrumentation files from the working tree.
- Merged dependency automation updates: `actions/upload-artifact` v4 → v7, `actions/download-artifact` v4 → v8.

---

## [0.2.8] — 2026-04-09

### Fixed (CI / Build)

- **Removed deprecated `macos-13` runner**: replaced with `macos-latest` (ARM) across `ci.yml` and `release.yml`. The Intel x86_64 binary is still produced via cross-compilation (`--target x86_64-apple-darwin`) on the ARM runner, which natively includes Apple's Intel SDK. GitHub has announced deprecation of `macos-13` runners; continuing to reference them caused CI startup failures.
- **`release.yml` trigger**: added `on: push: tags: ['v*']` so releases are now created automatically on every version tag push. Previously the workflow was `workflow_dispatch`-only, requiring a manual trigger for every release.
- **Publish condition**: unified `publish-github-release` and `publish-crates-io` job conditions to `${{ github.event_name == 'push' || github.event.inputs.dry_run == 'false' }}`. This safely handles both tag-triggered (automatic) and manual dry_run=false runs without false positives.

### Fixed (UX)

- **`keys import` error message when no `CONTEXT7_API=` key found**: the `anyhow` error chain was producing a redundant `Caused by:` line repeating the same message without the file path (e.g., `Error: No CONTEXT7_API= key found in: /path/file` followed by `Caused by: No CONTEXT7_API= key found in:`). Fixed by removing the redundant `.with_context()` wrapper in `cmd_keys_import` — the error from `extrair_chaves_env` already carries the full message via `bail!(t(...))`, so wrapping it again produced a truncated duplicate. The output is now a single clean error line.

### Added (Packaging / CI)

- **`build-flatpak` job** in `release.yml`: validates the Flatpak manifest (`packaging/flatpak/br.com.daniloaguiar.context7-cli.yaml`) on every release by running `flatpak-builder` against the vendored Rust sources. Ensures Flatpak packaging is continuously verified and not left to drift.
- **`build-snap` job** in `release.yml`: validates the Snap manifest (`packaging/snap/snapcraft.yaml`) on every release using `canonical/action-build@v1` (the official Canonical GitHub Action). Produces a `.snap` artifact retained for 7 days per run.
- **`docs/CROSS_PLATFORM.md`**: new dedicated document covering per-platform installation, behavior differences, and troubleshooting — Windows UTF-8 and ANSI color setup, macOS Gatekeeper (adhoc vs Developer ID codesign), Linux FHS paths, Alpine musl considerations, Flatpak sandboxing, Snap confinement, and SELinux transparency.

### Added (Input Validation)

- **Empty API key rejection**: `keys add` now rejects empty strings immediately with a clear error message and exit code 1 — previously an empty key would be silently stored, causing opaque authentication failures on every subsequent request.
- **Format warning for keys without `ctx7sk-` prefix**: `keys add` emits a non-blocking `stderr` warning when the supplied key does not start with the expected `ctx7sk-` prefix (e.g., keys copied with extra whitespace or from a wrong source). The key is still stored to allow future format changes; the warning is informational only and does not affect exit code.
- **`keys list --json` flag**: `keys list` now supports the global `--json` flag, outputting the list of masked keys as a JSON array. This enables scripting and automation (e.g., `context7 keys list --json | jaq 'length'`).
- **`CONTEXT7_HOME` path traversal logging**: the existing path traversal rejection in `resolver_home_override` now emits a `tracing::warn` diagnostic instead of failing silently, making it easier to diagnose misconfigured `CONTEXT7_HOME` values.

### Technical

- **194 tests passing, 0 failing** (up from 190 in v0.2.7 — 4 new regression tests added for input validation).
- **Zero CVEs**, `cargo deny` PASS on 4 checks (advisories, licenses, bans, sources).
- **All 9 canonical decisions preserved**: rand 0.8, colored 2, thiserror 2, directories 6, rust-version 1.75, 842-byte anti-bot hook, `added_at: String`, `stars: i64 -1`, crate-vs-binary names.
- **Docs fix**: corrected `keys list` output examples in `docs/HOW_TO_USE.md` — timestamps now shown as `YYYY-MM-DD HH:MM:SS` (matching the display introduced in v0.2.6) rather than the legacy RFC3339 format with nanosecond precision.

---

## [0.2.7] — 2026-04-09

### Fixed (CI / Build)

- **MSRV CI job**: pinned `assert_cmd = "=2.1.2"` in `[dev-dependencies]` (last version using `edition = "2021"` before the crate migrated to `edition = "2024"` requiring Rust 1.85). Restores MSRV 1.75 compatibility while preserving the canonical decision from Session 09.
- **Coverage gate**: `cargo llvm-cov` now ignores structural dispatcher files (`src/cli.rs`, `src/lib.rs`, `src/main.rs`) via `--ignore-filename-regex`. These files contain async dispatchers and entry points with ~0% coverage by design, and were dragging the 80% total threshold down below 79%. Business-logic files (`src/storage.rs` 95%, `src/errors.rs` 94%, `src/output.rs`, `src/i18n.rs`) continue to be fully validated.
- **Release workflow startup failure**: unified the two Apple codesign steps (adhoc vs Developer ID) into a single step with secrets injected via `env:` and shell-level `if`. The previous `if: ${{ secrets.APPLE_TEAM_ID == '' }}` conditions referenced the `secrets` context in a step-level `if:`, which is **not allowed** by GitHub Actions (see context availability docs). This was causing the release workflow to fail at startup in 0s on every push to `main`.

### Technical

- **Zero runtime changes**: the binary behavior is **identical** to v0.2.6. Users already on v0.2.6 gain nothing by upgrading functionally — this release exists solely to restore a green CI pipeline.
- **Zero runtime dependency changes**: only the `[dev-dependencies]` section was touched (`assert_cmd` pin).
- **190 tests passing, 0 failing** (unchanged from v0.2.6).
- **Zero CVEs**, `cargo deny` PASS on 4 checks (advisories, licenses, bans, sources).
- **All 9 canonical decisions preserved**: rand 0.8, colored 2, thiserror 2, directories 6, rust-version 1.75, 842-byte anti-bot hook, `added_at: String`, `stars: i64 -1`, crate-vs-binary names.

---

## [0.2.6] — 2026-04-09

### Fixed

- **`keys remove` exit code on failure**: `cmd_keys_remove` now returns `Err(ErroContext7::OperacaoKeysFalhou)` (exit code 1) when the index is invalid (0 or out of range) or when the storage is empty. Previously all three error paths returned `Ok(())`, silently reporting success (exit code 0) — a regression spotted in v0.2.4 QA. The user-facing colored message is unchanged; only the process exit code is corrected.
- **"No API key configured" message hardcoded in Portuguese**: `carregar_chaves_api` in `storage.rs` now calls `t(Mensagem::NenhumaChaveConfigurada)` instead of a hardcoded Portuguese string, so the error message correctly respects the `--lang` flag and `CONTEXT7_LANG` environment variable.
- **`keys list` timestamp display**: timestamps were rendered as full RFC3339 strings with 9-digit nanosecond precision (e.g., `2026-04-09T13:34:59.060818734+00:00`). Now formatted as `YYYY-MM-DD HH:MM:SS` (e.g., `2026-04-09 13:34:59`) via the new `formatar_added_at_display` helper in `output.rs`. The on-disk `added_at` format is preserved (canonical RFC3339 — display-only change).

### Security

- **Path traversal regression test** (`LOW-01`): added `testa_context7_home_rejeita_path_traversal` covering `..`, `../../../etc`, and `/tmp/../etc`. Confirms that `resolver_home_override` rejects all `Component::ParentDir` paths and falls back to XDG defaults. The rejection logic was already in place since v0.2.5; this test makes it explicit and prevents regression.

### Technical

- 173 tests passing (up from 172 in v0.2.5).
- Zero breaking changes — all subcommand behavior, storage formats, and public API remain identical.
- `formatar_added_at_display` is a new public function in `output.rs`, testable in isolation.
- Documentation corrected: `CONTEXT7_HOME` creates missing directories automatically on first use (was documented as silent XDG fallback).

---

## [0.2.5] — 2026-04-09

### Fixed

- **CI matrix — MSRV job**: the `cargo check` step now regenerates `Cargo.lock` using the 1.75 toolchain before running checks, fixing a build failure caused by the v4 lockfile format being incompatible with Rust 1.75.
- **Storage tests leaking state on Windows/macOS**: 18 tests in `storage::testes` that resolved `directories::ProjectDirs` to real user paths now use `CONTEXT7_HOME` env-var override for full isolation — no more cross-test contamination on non-Linux CI runners.
- **`commit-check` job used `grep -iE`**: replaced with `rg -qi` / `rg -i` to comply with the project's CLI tools policy (no `grep` allowed).

### Added

- **`CONTEXT7_HOME` environment variable** — set a custom directory for the `context7` config file, overriding the default XDG path. Useful for dotfiles, NixOS configs, Docker containers, and isolated test environments. Precedence: `CONTEXT7_HOME` > `ProjectDirs` default.
- **Alpine musl CI job**: validates that the binary builds correctly on musl libc (via `x86_64-unknown-linux-musl` target).
- **Universal Binary for macOS** (`release.yml`): runs `lipo` to combine `x86_64-apple-darwin` and `aarch64-apple-darwin` slices; applies ad-hoc code signing; optionally uses `notarytool` when credentials are available.
- **Flatpak manifest** at `packaging/flatpak/` for distribution via Flathub.
- **Snap manifest** (`packaging/snap/snapcraft.yaml`) for distribution via the Snap Store.
- **CI jobs**: `cargo-audit` (vulnerability scan), `cargo deny check` (license + advisory), `cargo-llvm-cov` (coverage gate at 80% lines).
- **Prebuilt artifacts** uploaded to each GitHub Release: `tar.gz` for Linux (gnu + musl), `.zip` for Windows, Universal Binary for macOS.

### Changed

- **`dependency-update config`**: added `ignore` entries for `rand 0.9.x`, `colored 3.x`, `toml 2.x`, `actions/checkout 5.x`/`6.x`, and `dtolnay/rust-toolchain 1.100` — all major-bump versions that would violate MSRV or break canonical API decisions.

### Security

- Closed 4 dependency automation PRs (`#1` dtolnay toolchain bump, `#2` actions/checkout v5, `#3` colored 3.x, `#4` rand 0.9.x) — all major upgrades that would violate MSRV or break the project's canonical dependency decisions.

---

## [0.2.4] — 2026-04-09

### Fixed

- **Windows UTF-8 output**: enabled UTF-8 console mode on Windows via `SetConsoleOutputCP(65001)` at startup so that accented characters and box-drawing symbols render correctly in CMD and PowerShell.
- **Windows ANSI colors**: enabled Virtual Terminal Processing via `SetConsoleMode` so that `colored` escape sequences render as colors instead of raw codes on Windows consoles.
- **`keys clear` confirmation**: the interactive prompt now accepts both `y` and `yes` (case-insensitive), fixing a usability issue where typing `yes` was rejected.
- **`keys remove 0` message**: the 1-based index check now shows a clear error ("index must be ≥ 1") instead of silently failing with a generic message.

### Added

- **CI pipeline** (`ci.yml`): matrix build on `ubuntu-latest`, `windows-latest`, `macos-latest` for Rust stable; runs `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check`, and `cargo test`.
- **Release workflow** (`release.yml`): triggered on `v*` tags; publishes to crates.io and creates a GitHub Release.
- **dependency automation** (`dependency-update config`): weekly Cargo dependency updates.
- **PR template** (`.github/PULL_REQUEST_TEMPLATE.md`): bilingual checklist reminding contributors to use "Squash and merge" for bot PRs.
- **`deny.toml`**: `cargo-deny` configuration for license allow-list and advisory database checks.

### Docs

- **Step 4 output format** (HOW_TO_USE.md): updated all examples to match the v0.2.3 format — library title in bold, ID dimmed on the line below, trust score inline as `(trust X.X/10)`.
- **Retry count** (HOW_TO_USE.md, context7-llm-rules.md): corrected all references from "3 attempts/retries" to "5 attempts" to match the actual `MAX_TENTATIVAS = 5` constant.
- **Step 3 keys list format** (HOW_TO_USE.md): updated the `keys list` example to the current `[N]  ctx7sk-...  (added: ISO-timestamp)` format.
- **Library ID `/reactjs/react.dev`** (all docs): replaced all 60 occurrences of the stale `/facebook/react` ID across `HOW_TO_USE.md`, `README.md`, `context7-skill.md`, and `context7-llm-rules.md`.
- **Anchor fix** (HOW_TO_USE.md): corrected a broken link pointing to a renamed heading.
- **LLM rules link** (HOW_TO_USE.md): the "LLM rules" reference now links to `docs/context7-llm-rules.md` (was linking to a stale `docs/llm-rules.md` path).
- **Stale v0.2.0 troubleshooting note** (HOW_TO_USE.md §8): replaced with an instruction to upgrade via `cargo install context7-cli --force`.
- **TOC** (HOW_TO_USE.md): added a bilingual Table of Contents with anchor links to each major section.
- **`context7-skill.md` version bump**: `0.2.3` → `0.2.4`.
- **Cargo.toml description**: improved to reflect the bilingual and multi-key nature of the tool.

### Changed

- **User agent**: bumped from `context7-cli/0.2.3` to `context7-cli/0.2.4`.

### Dependencies

- Bump `reqwest` from 0.12.8 to 0.12.9
- Bump `chrono` from 0.4.38 to 0.4.44
- Bump `directories` from 5.x to 6.0.0
- Bump `toml` from 0.8.x to 1.1.2

---

## [0.2.3] — 2026-04-09

### Added

- **Library-not-found UX hint**: when `context7 docs <id>` fails with HTTP 404, the CLI now prints a yellow hint suggesting `context7 library <name>` to look up the correct ID — in addition to the existing structured error message.
- **`exibir_dica_biblioteca_nao_encontrada()`** — new public function in `output.rs` that renders the hint using `colored::Colorize`.

### Changed

- **`--help` parameter names in English**: renamed clap identifiers from Portuguese to English for a better experience for non-PT users: `<NOME>` → `<NAME>`, `<CHAVE>` → `<KEY>`, `<INDICE>` → `<INDEX>`, `<ARQUIVO>` → `<FILE>`.
- **Library list format**: `exibir_bibliotecas_formatado` now renders title in bold (primary focus), library ID in dimmed below it, and trust score inline as `(trust X.X/10)` instead of a separate `Trust:` label on its own line.
- **Trust score label**: `"Trust:"` → `"trust"` (lowercase, no colon — displayed inline inside parentheses).
- **README tagline**: updated to reflect the single-binary, zero-runtime nature of the tool more concisely.
- **User agent**: bumped from `context7-cli/0.2.2` to `context7-cli/0.2.3`.

---

## [0.2.2] — 2026-04-09

### Fixed

- **`docs` subcommand UX for missing libraries**: HTTP 404 responses from the Context7 API used to be reported as `No valid API key available after 5 attempts`, misleading users into thinking their keys were invalid. Now surfaces a dedicated `BibliotecaNaoEncontrada` error with a clear message suggesting `context7 library <name>` to look up the correct ID.
- **`executar_com_retry` short-circuits on 404**: the retry loop now aborts immediately when a library is not found (no point trying other keys).
- **Documentation stale references**: removed 19 legacy `trust_score` (snake_case) mentions across `docs/HOW_TO_USE.md`, `docs/context7-skill.md`, and `docs/context7-llm-rules.md` — the API returns `trustScore` (camelCase) since v0.2.1.
- **Documentation stale schema**: removed 6 references to `source_urls` and `content` (legacy snippet fields) in `docs/context7-skill.md` and `docs/context7-llm-rules.md`, replaced with the current schema (`codeId`, `codeTitle`, `codeDescription`, `codeList`, `pageTitle`, `relevance`, `model`).
- **Documentation references to removed command**: removed 6 references to `context7 keys rotate` across the 3 docs files (this command was deleted in v0.2.1 but still appeared in examples).
- **`context7-skill.md` frontmatter**: `version: 0.2.0` → `version: 0.2.2`.

### Added

- **`ErroContext7::BibliotecaNaoEncontrada`** — new structured error variant for HTTP 404 on the docs endpoint.
- **`Mensagem::BibliotecaNaoEncontradaApi`** — new i18n variant with EN + PT translations.
- Wiremock integration tests covering HTTP 404 handling and the `executar_com_retry` short-circuit.

### Changed

- **User agent**: bumped from `context7-cli/0.2.1` to `context7-cli/0.2.2`.

---

## [0.2.1] — 2026-04-09

### Fixed

- **`docs` subcommand schema**: `DocumentationSnippet` struct expected `content`, `type`, `source_urls` but the Context7 API returns `codeTitle`, `codeDescription`, `codeList`, `pageTitle`, `codeId`, `relevance`, `model` (camelCase). Schema now matches the real API response.
- **`--text` flag**: was always failing because the code called `.json()` on the plain-text response body. Now uses `.text().await` directly and prints the raw markdown.
- **`LibrarySearchResult.trust_score`**: was always `null` because the API returns `trustScore` (camelCase) but the struct expected `trust_score` (snake_case). Fixed with `#[serde(rename_all = "camelCase")]`.
- **Retry logic**: was artificially limited to 3 attempts via `3usize.min(chaves.len())`. Now uses up to 5 keys. Also short-circuits when the response is HTTP 200 with a parse error (schema issue, not a key issue).
- **Error messages i18n**: now respect the `--lang`/`CONTEXT7_LANG` setting (previously hardcoded in Portuguese).

### Added

- **`LibrarySearchResult`** now exposes `stars`, `total_snippets`, `total_tokens`, `verified`, `branch`, `state` from the API.
- **`buscar_documentacao_texto`**: new public function in `api.rs` for raw text output.

### Removed

- **`context7 keys rotate`** subcommand — rotation has always been automatic (random shuffle per request); the explicit command was dead code that was only referenced in docs, never implemented.

### Changed

- **User agent**: bumped from `context7-cli/0.2.0` to `context7-cli/0.2.1`.

---

## [0.2.0] — 2026-04-09

### Added

- **Bilingual runtime UI** — English and Brazilian Portuguese output with auto-detect via system locale (`sys-locale` crate). Override at runtime with `--lang en` or `--lang pt`, or permanently with `CONTEXT7_LANG` environment variable.
- **Public library API** — `context7_cli` crate now published on [docs.rs](https://docs.rs/context7-cli). Programmatic usage is now possible by adding `context7-cli` as a dependency. Public API docstrings written in English (Rust ecosystem convention).
- **`[lib]` target in Cargo.toml** — resolves the docs.rs build failure from v0.1.0 (`error: no library targets found in package`, build #3116184). The `src/lib.rs` module now exposes the public API.
- **`[package.metadata.docs.rs]` section** — proper feature documentation generation on docs.rs.
- **`docs/HOW_TO_USE.md`** — complete bilingual step-by-step guide covering Linux, macOS, and Windows (replaces `como_usa.md`).
- **`docs/context7-skill.md`** — LLM skill file for invoking `context7-cli` from AI assistants. Includes YAML frontmatter, usage patterns, output parsing guide, and error handling.
- **`docs/context7-llm-rules.md`** — 7 prescriptive rules and 5 prompt templates for LLM agents using `context7-cli`.
- **`context7 keys rotate`** — new key operation that manually advances the active key pointer to the next available key in the pool.
- **`-q` short flag** — short alias for `--query` in both the `library` and `docs` subcommands.
- **`CONTEXT7_HOME` environment variable** — alternative XDG base directory for config (primarily useful for testing and CI environments).
- **New dependency** — `sys-locale 0.3` for language auto-detect.
- **New dev dependencies** — `assert_cmd 2` and `predicates 3` for CLI integration tests.
- **`rust-version = "1.75"`** declared in Cargo.toml (MSRV).
- **Integration tests** in `tests/` directory using `assert_cmd` and `predicates`.

### Changed

- **Modular architecture** — refactored from monolithic `context7.rs` (3006 lines) to `src/` with 8 files:
  - `src/lib.rs` — public API exports
  - `src/main.rs` — binary entry point
  - `src/cli.rs` — clap argument parsing
  - `src/api.rs` — HTTP client and Context7 API calls
  - `src/storage.rs` — XDG config persistence
  - `src/output.rs` — colored/JSON/text formatting
  - `src/errors.rs` — structured error types with `thiserror`
  - `src/i18n.rs` — bilingual message lookup table
- **Public API docstrings** — now in English to match docs.rs audience expectations (internal comments and user-facing messages remain in Brazilian Portuguese).
- **README.md** — complete rewrite with bilingual structure (English first, then Português). Added "Why use it?" section with 8 benefits, quick install for 3 OS, and links to new docs files.

### Fixed

- **docs.rs build failure** — v0.1.0 build #3116184 failed with `error: no library targets found in package`. Resolved by adding `[lib]` target in Cargo.toml.
- **`como_usa.md` consolidation** — content migrated and improved in `docs/HOW_TO_USE.md` with bilingual structure and 3-OS coverage.

### Preserved

All v0.1.0 CLI behavior is preserved without breaking changes:
- Subcommands: `library`, `docs`, `keys`
- Aliases: `lib`, `search`, `doc`, `context`, `key`
- XDG storage layout (Linux/macOS/Windows)
- Key file permissions (`0o600` on Unix)
- API key hierarchy (env var → XDG → `.env` → compile-time)
- Retry logic (3 attempts, 500ms → 1s → 2s backoff)
- `--json` and `--text` flags
- All v0.1.0 `keys` suboperations: `add`, `list`, `remove`, `clear`, `path`, `import`, `export` (plus new `rotate`, which was later removed in v0.2.1 — rotation is automatic)

---

## [0.1.0] — 2026-04-08

### Added

- Initial release published on [crates.io](https://crates.io/crates/context7-cli) and [GitHub](https://github.com/daniloaguiarbr/context7-cli).
- Native Rust binary `context7` — single file, no runtime dependencies.
- Subcommand `library` (aliases: `lib`, `search`) — search Context7 libraries by name with optional semantic context.
- Subcommand `docs` (aliases: `doc`, `context`) — fetch library documentation by ID with `--query`, `--text`, `--json` flags.
- Subcommand `keys` (alias: `key`) — manage API keys locally via operations `add`, `list`, `remove`, `clear`, `path`, `import`, `export`.
- XDG-compliant config storage (`~/.config/context7/config.toml` on Linux) with `chmod 600` on Unix.
- Multi-key rotation with shuffle without replacement.
- Exponential backoff retry: 3 attempts, 500ms → 1s → 2s.
- Dual logging: stderr with ANSI colors + XDG state file with rotation-by-deletion.
- `--json` global flag for machine-readable output.
- `como_usa.md` — usage guide in Brazilian Portuguese (791 lines).
- License: MIT OR Apache-2.0.

---

## Português

---

## [0.5.2] — 2026-07-15 — Português

### Alterado

- **Migração de ownership / repositório**: owner do crate no crates.io passa a ser `danilo-aguiar-br` (conta anterior virou `ghost_405118`).
- **Repositório canônico no GitHub** movido para `https://github.com/danilo-aguiar-br/context7-cli` (`repository` / `homepage` no `Cargo.toml`).
- **Removido todo CI/CD de GitHub Actions** (`.github/workflows/`, dependency automation). Releases são publicadas manualmente; Actions **não** são recriadas no repo novo.
- E-mail de contato alinhado a `daniloaguiarbr@proton.me`.
- Versão do packaging snap alinhada a `0.5.2`.

### Notas

- O crates.io só atualiza `repository`/`homepage` ao publicar uma **nova versão**; este release carrega esses metadados.
- O trabalho local de `0.5.1` (health, quiet, etc.) está nesta árvore; `0.5.1` já estava no crates.io pela conta anterior.

---

## [0.3.0] — 2026-04-09

### Corrigido

- **`keys list --json` agora retorna `added_at` em formato legível**: antes exibia RFC3339 bruto com nanossegundos (`2026-04-09T18:02:45.943772067+00:00`), agora retorna `2026-04-09 18:02:45` — consistente com a saída formatada e a documentação.
- **Prompt de `keys clear` agora respeita `--lang`**: o prompt interativo estava hardcoded em português; agora usa o sistema i18n (`Mensagem::ConfirmarRemoverTodas`) e exibe no idioma selecionado pelo usuário.

### Alterado (Arquitetura)

- **Todos os `println!` centralizados em `output.rs`**: removidas 8 chamadas diretas `println!`/`print!` de `storage.rs` (5) e `cli.rs` (3). Novas funções em `output.rs`: `exibir_json_array_vazio`, `exibir_json_bruto`, `exibir_caminho_config`, `exibir_chave_exportada`, `exibir_json_resultados`, `exibir_texto_plano`, `confirmar_clear`. `output.rs` é agora o único módulo responsável por toda I/O de terminal.

### Adicionado (Testes)

- **15 novos testes** (219 total, 0 falhando):
  - 3 testes de validação JSON (parse válido, campos esperados, `added_at` legível)
  - 2 testes de CRLF (line endings Windows em arquivos `.env`)
  - 8 testes de i18n (EN/PT para `keys add`, `keys remove`, `keys clear --yes`, erro de chave API)
  - 2 testes cross-platform (rejeição de path traversal, saída UTF-8 em português)

### Adicionado (Documentação)

- **README.md completamente reescrito**: novas seções para Gerenciamento de Chaves, Variáveis de Ambiente, Padrões de Integração, Referência Rápida (cheat sheet) e Resolução de Problemas. Removidas notas de release infladas (movidas para CHANGELOG).
- **CROSS_PLATFORM.md expandido**: guias de instalação para NixOS e GNU Guix, documentação do isolamento de chaves no Flatpak, tabela comparativa Flatpak vs Snap vs Cargo.

### Adicionado (CI)

- **Smoke test binário musl**: `ci.yml` agora compila e executa o binário Alpine/musl (`--version` + `--help`) em vez de apenas compilar metadados.
- **Smoke test Universal Binary**: `release.yml` agora executa `--version` + `--help` no binário macOS Universal combinado via lipo antes de publicar.

---

## [0.2.9] — 2026-04-09

### Corrigido

- **Removido import redundante `use serde_json;`** em `tests/cli_integration.rs` que disparava warning `clippy::single_component_path_imports` com `-D warnings`, causando falha de CI nas três plataformas (ubuntu, macos, windows) e bloqueando o workflow de release.

### Adicionado (Docs)

- **Documentação `keys list --json`** em `docs/HOW_TO_USE.md` (EN + PT) — o flag `--json` para `keys list` foi adicionado na v0.2.8 mas não estava documentado no guia de uso.

### Técnico

- Limpeza de 93 arquivos `.profraw` residuais de instrumentação.
- Merge de atualizações dependency automation: `actions/upload-artifact` v4 → v7, `actions/download-artifact` v4 → v8.

---

## [0.2.8] — 2026-04-09

### Corrigido (CI / Build)

- **Runner `macos-13` descontinuado removido**: substituído por `macos-latest` (ARM) em `ci.yml` e `release.yml`. O binário Intel x86_64 continua sendo produzido por cross-compilação (`--target x86_64-apple-darwin`) no runner ARM, que inclui nativamente o SDK Intel da Apple. A GitHub anunciou a descontinuação dos runners `macos-13`; mantê-los causava falhas de inicialização no CI.
- **Trigger do `release.yml`**: adicionado `on: push: tags: ['v*']` para que releases sejam criadas automaticamente a cada push de tag de versão. Anteriormente o workflow era apenas `workflow_dispatch`, exigindo trigger manual para cada release.
- **Condição de publicação**: unificada a condição dos jobs `publish-github-release` e `publish-crates-io` para `${{ github.event_name == 'push' || github.event.inputs.dry_run == 'false' }}`. Cobre corretamente tanto runs automáticos por tag quanto manuais com dry_run=false sem falsos positivos.

### Corrigido (UX)

- **Mensagem de erro em `keys import` quando nenhuma chave `CONTEXT7_API=` é encontrada**: a cadeia de erros do `anyhow` produzia uma linha `Caused by:` redundante repetindo a mesma mensagem sem o caminho do arquivo (ex.: `Error: Nenhuma chave CONTEXT7_API= encontrada em: /caminho` seguido de `Caused by: Nenhuma chave CONTEXT7_API= encontrada em:`). Corrigido removendo o `.with_context()` redundante em `cmd_keys_import` — o erro de `extrair_chaves_env` já carrega a mensagem completa via `bail!(t(...))`, então envolvê-lo novamente produzia um duplicado truncado. A saída agora é uma única linha de erro limpa.

### Adicionado (Packaging / CI)

- **Job `build-flatpak`** no `release.yml`: valida o manifesto Flatpak (`packaging/flatpak/br.com.daniloaguiar.context7-cli.yaml`) a cada release executando `flatpak-builder` com as fontes Rust em modo vendored. Garante que o packaging Flatpak seja verificado continuamente e não fique desatualizado.
- **Job `build-snap`** no `release.yml`: valida o manifesto Snap (`packaging/snap/snapcraft.yaml`) a cada release usando `canonical/action-build@v1` (a action oficial do Canonical). Produz um artefato `.snap` retido por 7 dias por execução.
- **`docs/CROSS_PLATFORM.md`**: novo documento dedicado cobrindo instalação, diferenças de comportamento e troubleshooting por plataforma — configuração UTF-8 e cores ANSI no Windows, Gatekeeper do macOS (codesign adhoc vs Developer ID), caminhos FHS no Linux, considerações sobre musl Alpine, sandboxing do Flatpak, confinamento do Snap e transparência SELinux.

### Adicionado (Validação de Entrada)

- **Rejeição de chave de API vazia**: `keys add` agora rejeita strings vazias imediatamente com uma mensagem de erro clara e exit code 1 — anteriormente uma chave vazia era armazenada silenciosamente, causando falhas de autenticação opacas em todas as requisições subsequentes.
- **Aviso de formato para chaves sem o prefixo `ctx7sk-`**: `keys add` emite um aviso não-bloqueante no `stderr` quando a chave fornecida não começa com o prefixo esperado `ctx7sk-` (ex.: chaves copiadas com espaço extra ou de uma fonte errada). A chave ainda é armazenada para permitir mudanças futuras de formato; o aviso é apenas informativo e não afeta o exit code.
- **Flag `--json` em `keys list`**: `keys list` agora suporta a flag global `--json`, produzindo a lista de chaves mascaradas como um array JSON. Isso habilita scripts e automação (ex.: `context7 keys list --json | jaq 'length'`).
- **Log de rejeição de path traversal em `CONTEXT7_HOME`**: a rejeição de path traversal existente em `resolver_home_override` agora emite um diagnóstico `tracing::warn` em vez de falhar silenciosamente, facilitando o diagnóstico de valores mal configurados de `CONTEXT7_HOME`.

### Técnico

- **194 testes passando, 0 falhando** (ante 190 na v0.2.7 — 4 novos testes de regressão adicionados para validação de entrada).
- **Zero CVEs**, `cargo deny` PASS nos 4 checks (advisories, licenças, bans, sources).
- **Todas as 9 decisões canônicas preservadas**: rand 0.8, colored 2, thiserror 2, directories 6, rust-version 1.75, hook anti-bot 842 bytes, `added_at: String`, `stars: i64 -1`, nomes crate-vs-binary.
- **Correção de documentação**: exemplos de output de `keys list` em `docs/HOW_TO_USE.md` atualizados — timestamps agora exibidos como `YYYY-MM-DD HH:MM:SS` (conforme introduzido na v0.2.6) em vez do formato RFC3339 legado com precisão de nanossegundos.

---

## [0.2.7] — 2026-04-09

### Corrigido (CI / Build)

- **Job de CI MSRV**: `assert_cmd` pinado em `"=2.1.2"` em `[dev-dependencies]` (última versão usando `edition = "2021"` antes de a crate migrar para `edition = "2024"` exigindo Rust 1.85). Restaura compatibilidade com MSRV 1.75 preservando a decisão canônica da Sessão 09.
- **Gate de cobertura**: `cargo llvm-cov` agora ignora arquivos de dispatcher estrutural (`src/cli.rs`, `src/lib.rs`, `src/main.rs`) via `--ignore-filename-regex`. Esses arquivos contêm dispatchers async e entry points com ~0% de cobertura por design, e estavam puxando o threshold total de 80% para abaixo de 79%. Arquivos de lógica de negócio (`src/storage.rs` 95%, `src/errors.rs` 94%, `src/output.rs`, `src/i18n.rs`) continuam sendo totalmente validados.
- **Startup failure do workflow de release**: unificamos os dois steps de codesign da Apple (adhoc vs Developer ID) em um único step com secrets injetados via `env:` e `if` em shell. As condições anteriores `if: ${{ secrets.APPLE_TEAM_ID == '' }}` referenciavam o contexto `secrets` em `if:` de step, o que **não é permitido** pelo GitHub Actions (ver docs oficiais de context availability). Isso causava falha do workflow de release no startup, em 0s, a cada push para `main`.

### Técnico

- **Zero mudanças em runtime**: o comportamento do binário é **idêntico** ao v0.2.6. Usuários que já estão na v0.2.6 não ganham nada funcionalmente com o upgrade — este release existe apenas para restaurar um pipeline de CI verde.
- **Zero mudanças em dependências de runtime**: apenas a seção `[dev-dependencies]` foi tocada (pin do `assert_cmd`).
- **190 testes passando, 0 falhando** (inalterado em relação a v0.2.6).
- **Zero CVEs**, `cargo deny` PASS nos 4 checks (advisories, licenses, bans, sources).
- **Todas as 9 decisões canônicas preservadas**: rand 0.8, colored 2, thiserror 2, directories 6, rust-version 1.75, hook anti-bot 842 bytes, `added_at: String`, `stars: i64 -1`, nomes crate-vs-binário.

---

## [0.2.6] — 2026-04-09

### Corrigido

- **Exit code de `keys remove` em caso de falha**: `cmd_keys_remove` agora retorna `Err(ErroContext7::OperacaoKeysFalhou)` (exit code 1) quando o índice é inválido (0 ou fora do intervalo) ou quando o storage está vazio. Anteriormente os três caminhos de erro retornavam `Ok(())`, reportando sucesso silenciosamente (exit code 0) — regressão identificada no QA da v0.2.4. A mensagem colorida exibida ao usuário permanece inalterada; apenas o exit code foi corrigido.
- **Mensagem "Nenhuma chave de API configurada" hardcoded em português**: `carregar_chaves_api` em `storage.rs` agora usa `t(Mensagem::NenhumaChaveConfigurada)` em vez de uma string PT hardcoded, garantindo que a mensagem de erro respeite a flag `--lang` e a variável de ambiente `CONTEXT7_LANG`.
- **Exibição de timestamp em `keys list`**: os timestamps eram exibidos como strings RFC3339 completas com 9 dígitos de nanosegundos (ex.: `2026-04-09T13:34:59.060818734+00:00`). Agora formatados como `AAAA-MM-DD HH:MM:SS` (ex.: `2026-04-09 13:34:59`) pelo novo helper `formatar_added_at_display` em `output.rs`. O formato de disco de `added_at` é preservado (RFC3339 canônico — mudança apenas de exibição).

### Segurança

- **Teste de regressão para path traversal** (`LOW-01`): adicionado `testa_context7_home_rejeita_path_traversal` cobrindo `..`, `../../../etc` e `/tmp/../etc`. Confirma que `resolver_home_override` rejeita todos os caminhos com `Component::ParentDir` e cai no fallback XDG. A lógica de rejeição já existia desde a v0.2.5; este teste a torna explícita e previne regressões.

### Técnico

- 173 testes passando (ante 172 na v0.2.5).
- Zero breaking changes — comportamento dos subcomandos, formatos de storage e API pública permanecem idênticos.
- `formatar_added_at_display` é uma nova função pública em `output.rs`, testável de forma isolada.
- Documentação corrigida: `CONTEXT7_HOME` cria diretórios inexistentes automaticamente no primeiro uso (era documentado como fallback XDG silencioso).

---

## [0.2.5] — 2026-04-09

### Corrigido

- **CI matrix — job MSRV**: o passo `cargo check` agora regenera o `Cargo.lock` usando a toolchain 1.75 antes de rodar as verificações, corrigindo falha de build causada pelo formato v4 do lockfile ser incompatível com o Rust 1.75.
- **Testes de storage com vazamento de estado no Windows/macOS**: 18 testes em `storage::testes` que resolviam caminhos reais via `directories::ProjectDirs` agora usam a variável `CONTEXT7_HOME` para isolamento completo — sem mais contaminação entre testes nos runners de CI não-Linux.
- **Job `commit-check` usava `grep -iE`**: substituído por `rg -qi` / `rg -i` para cumprir a política de ferramentas CLI do projeto (proibido usar `grep`).

### Adicionado

- **Variável de ambiente `CONTEXT7_HOME`** — define um diretório customizado para o arquivo de configuração do `context7`, sobrepondo o caminho XDG padrão. Útil para dotfiles, configurações NixOS, contêineres Docker e ambientes de teste isolados. Precedência: `CONTEXT7_HOME` > padrão `ProjectDirs`.
- **Job Alpine musl no CI**: valida que o binário compila corretamente com musl libc (via target `x86_64-unknown-linux-musl`).
- **Universal Binary para macOS** (`release.yml`): executa `lipo` para combinar as slices `x86_64-apple-darwin` e `aarch64-apple-darwin`; aplica assinatura ad-hoc; usa `notarytool` quando as credenciais estão disponíveis.
- **Manifesto Flatpak** em `packaging/flatpak/` para distribuição via Flathub.
- **Manifesto Snap** (`packaging/snap/snapcraft.yaml`) para distribuição via Snap Store.
- **Jobs de CI**: `cargo-audit` (varredura de vulnerabilidades), `cargo deny check` (licenças + advisories), `cargo-llvm-cov` (gate de cobertura em 80% de linhas).
- **Artefatos pré-compilados** publicados em cada GitHub Release: `tar.gz` para Linux (gnu + musl), `.zip` para Windows, Universal Binary para macOS.

### Alterado

- **`dependency-update config`**: adicionadas entradas `ignore` para `rand 0.9.x`, `colored 3.x`, `toml 2.x`, `actions/checkout 5.x`/`6.x` e `dtolnay/rust-toolchain 1.100` — versões com major bump que violariam o MSRV ou quebrariam decisões canônicas de API.

### Segurança

- Fechados 4 PRs do dependency automation (`#1` bump dtolnay toolchain, `#2` actions/checkout v5, `#3` colored 3.x, `#4` rand 0.9.x) — todos upgrades major que violariam o MSRV ou quebrariam as decisões canônicas de dependências do projeto.

---

## [0.2.4] — 2026-04-09

### Corrigido

- **Saída UTF-8 no Windows**: habilitado modo UTF-8 no console Windows via `SetConsoleOutputCP(65001)` na inicialização, para que caracteres acentuados e símbolos sejam exibidos corretamente no CMD e PowerShell.
- **Cores ANSI no Windows**: habilitado Virtual Terminal Processing via `SetConsoleMode` para que as sequências de escape do `colored` sejam renderizadas como cores em vez de códigos brutos no console Windows.
- **Confirmação no `keys clear`**: o prompt interativo agora aceita `y` e `yes` (sem distinção de maiúsculas/minúsculas), corrigindo um problema em que digitar `yes` era rejeitado.
- **Mensagem de `keys remove 0`**: a verificação de índice 1-based agora exibe uma mensagem clara ("o índice deve ser ≥ 1") em vez de falhar silenciosamente com mensagem genérica.

### Adicionado

- **Pipeline de CI** (`ci.yml`): build em matriz em `ubuntu-latest`, `windows-latest`, `macos-latest` com Rust estável; executa `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt --check` e `cargo test`.
- **Workflow de release** (`release.yml`): acionado em tags `v*`; publica no crates.io e cria um GitHub Release.
- **dependency automation** (`dependency-update config`): atualizações semanais de dependências Cargo.
- **Template de PR** (`.github/PULL_REQUEST_TEMPLATE.md`): checklist bilíngue lembrando contribuidores de usar "Squash and merge" para PRs de bots.
- **`deny.toml`**: configuração do `cargo-deny` para lista de licenças permitidas e verificação de advisories de segurança.

### Documentação

- **Formato de saída do Passo 4** (HOW_TO_USE.md): todos os exemplos atualizados para o formato v0.2.3 — título da biblioteca em negrito, ID em dimmed na linha abaixo, trust score inline como `(confiança X.X/10)`.
- **Contagem de tentativas** (HOW_TO_USE.md, context7-llm-rules.md): corrigidas todas as referências de "3 tentativas/retries" para "5 tentativas" para corresponder à constante real `MAX_TENTATIVAS = 5`.
- **Formato da lista de chaves no Passo 3** (HOW_TO_USE.md): exemplo de `keys list` atualizado para o formato atual `[N]  ctx7sk-...  (added: ISO-timestamp)`.
- **ID de biblioteca `/reactjs/react.dev`** (todos os docs): substituídas 60 ocorrências do ID obsoleto `/facebook/react` em `HOW_TO_USE.md`, `README.md`, `context7-skill.md` e `context7-llm-rules.md`.
- **Correção de âncora** (HOW_TO_USE.md): corrigido link quebrado apontando para um heading renomeado.
- **Link para LLM rules** (HOW_TO_USE.md): a referência "LLM rules" agora aponta para `docs/context7-llm-rules.md` (estava apontando para o caminho obsoleto `docs/llm-rules.md`).
- **Nota obsoleta de troubleshooting v0.2.0** (HOW_TO_USE.md §8): substituída por instrução de upgrade via `cargo install context7-cli --force`.
- **Sumário** (HOW_TO_USE.md): adicionado sumário bilíngue com links de âncora para cada seção principal.
- **Bump de versão do `context7-skill.md`**: `0.2.3` → `0.2.4`.
- **Descrição do Cargo.toml**: melhorada para refletir a natureza bilíngue e de múltiplas chaves da ferramenta.

### Alterado

- **User agent**: atualizado de `context7-cli/0.2.3` para `context7-cli/0.2.4`.

### Dependências

- Atualização do `reqwest` de 0.12.8 para 0.12.9
- Atualização do `chrono` de 0.4.38 para 0.4.44
- Atualização do `directories` de 5.x para 6.0.0
- Atualização do `toml` de 0.8.x para 1.1.2

---

## [0.2.3] — 2026-04-09

### Adicionado

- **Dica UX para biblioteca não encontrada**: quando `context7 docs <id>` falha com HTTP 404, a CLI agora exibe uma dica em amarelo sugerindo `context7 library <nome>` para consultar o ID correto — além da mensagem de erro estruturada já existente.
- **`exibir_dica_biblioteca_nao_encontrada()`** — nova função pública em `output.rs` que renderiza a dica usando `colored::Colorize`.

### Alterado

- **Nomes de parâmetros no `--help` em inglês**: renomeados os identificadores clap de português para inglês para melhor experiência de usuários não-PT: `<NOME>` → `<NAME>`, `<CHAVE>` → `<KEY>`, `<INDICE>` → `<INDEX>`, `<ARQUIVO>` → `<FILE>`.
- **Formato da lista de bibliotecas**: `exibir_bibliotecas_formatado` agora renderiza o título em negrito (foco principal), o ID da biblioteca em dimmed abaixo, e o trust score inline como `(confiança X.X/10)` em vez de uma label `Confiança:` separada.
- **Label de trust score**: `"Confiança:"` → `"confiança"` (minúsculo, sem dois-pontos — exibido inline entre parênteses).
- **Tagline do README**: atualizada para refletir de forma mais concisa a natureza single-binary e zero-runtime da ferramenta.
- **User agent**: atualizado de `context7-cli/0.2.2` para `context7-cli/0.2.3`.

---

## [0.2.2] — 2026-04-09

### Corrigido

- **UX do subcomando `docs` para bibliotecas inexistentes**: respostas HTTP 404 da API Context7 eram reportadas como `Nenhuma chave de API válida disponível após 5 tentativas`, fazendo o usuário pensar que as chaves estavam inválidas. Agora expõe um erro dedicado `BibliotecaNaoEncontrada` com mensagem clara sugerindo `context7 library <nome>` para consultar o ID correto.
- **`executar_com_retry` aborta imediatamente em 404**: o loop de retry agora para logo quando uma biblioteca não é encontrada (não adianta tentar outra chave).
- **Referências stale na documentação**: removidas 19 ocorrências legadas de `trust_score` (snake_case) em `docs/HOW_TO_USE.md`, `docs/context7-skill.md` e `docs/context7-llm-rules.md` — a API retorna `trustScore` (camelCase) desde a v0.2.1.
- **Schema stale na documentação**: removidas 6 referências a `source_urls` e `content` (campos legados de snippet) em `docs/context7-skill.md` e `docs/context7-llm-rules.md`, substituídas pelo schema atual (`codeId`, `codeTitle`, `codeDescription`, `codeList`, `pageTitle`, `relevance`, `model`).
- **Referências ao comando removido**: eliminadas 6 menções a `context7 keys rotate` nos 3 arquivos de docs (este comando foi deletado na v0.2.1 mas ainda aparecia nos exemplos).
- **Frontmatter do `context7-skill.md`**: `version: 0.2.0` → `version: 0.2.2`.

### Adicionado

- **`ErroContext7::BibliotecaNaoEncontrada`** — nova variante estruturada para HTTP 404 no endpoint de docs.
- **`Mensagem::BibliotecaNaoEncontradaApi`** — nova variante i18n com traduções EN + PT.
- Testes de integração com wiremock cobrindo o handling de HTTP 404 e o short-circuit no `executar_com_retry`.

### Alterado

- **User agent**: atualizado de `context7-cli/0.2.1` para `context7-cli/0.2.2`.

---

## [0.2.1] — 2026-04-09

### Corrigido

- **Schema do subcomando `docs`**: a struct `DocumentationSnippet` esperava `content`, `type`, `source_urls` mas a API do Context7 retorna `codeTitle`, `codeDescription`, `codeList`, `pageTitle`, `codeId`, `relevance`, `model` (camelCase). Schema agora reflete a resposta real da API.
- **Flag `--text`**: sempre falhava porque o código chamava `.json()` no corpo de texto plano. Agora usa `.text().await` diretamente e imprime o markdown bruto.
- **`LibrarySearchResult.trust_score`**: era sempre `null` porque a API retorna `trustScore` (camelCase) mas o struct esperava `trust_score` (snake_case). Corrigido com `#[serde(rename_all = "camelCase")]`.
- **Lógica de retry**: estava artificialmente limitada a 3 tentativas via `3usize.min(chaves.len())`. Agora usa até 5 chaves. Também aborta imediatamente quando a resposta é HTTP 200 com erro de parse (problema de schema, não de chave).
- **Mensagens de erro i18n**: agora respeitam a configuração `--lang`/`CONTEXT7_LANG` (antes eram hardcoded em português).

### Adicionado

- **`LibrarySearchResult`** agora expõe `stars`, `total_snippets`, `total_tokens`, `verified`, `branch`, `state` da API.
- **`buscar_documentacao_texto`**: nova função pública em `api.rs` para saída de texto bruto.

### Removido

- **Subcomando `context7 keys rotate`** — rotação sempre foi automática (shuffle aleatório por requisição); o comando explícito era código morto que só era referenciado na documentação, nunca implementado.

### Alterado

- **User agent**: atualizado de `context7-cli/0.2.0` para `context7-cli/0.2.1`.

---

## [0.2.0] — 2026-04-09

### Adicionado

- **Interface bilíngue em runtime** — saída em inglês e português brasileiro com auto-detect via locale do sistema (crate `sys-locale`). Override em runtime com `--lang en` ou `--lang pt`, ou permanentemente com a variável de ambiente `CONTEXT7_LANG`.
- **API pública de biblioteca** — o crate `context7_cli` agora está publicado no [docs.rs](https://docs.rs/context7-cli). Uso programático agora é possível adicionando `context7-cli` como dependência. Docstrings da API pública escritas em inglês (convenção do ecossistema Rust).
- **Target `[lib]` no Cargo.toml** — resolve a falha de build no docs.rs da v0.1.0 (`error: no library targets found in package`, build #3116184). O módulo `src/lib.rs` agora expõe a API pública.
- **Seção `[package.metadata.docs.rs]`** — geração correta de documentação de features no docs.rs.
- **`docs/HOW_TO_USE.md`** — guia passo a passo bilíngue completo cobrindo Linux, macOS e Windows (substitui `como_usa.md`).
- **`docs/context7-skill.md`** — arquivo de skill para LLMs invocarem o `context7-cli` a partir de assistentes de IA. Inclui frontmatter YAML, padrões de uso, guia de parsing de output e tratamento de erros.
- **`docs/context7-llm-rules.md`** — 7 regras prescritivas e 5 templates de prompt para agentes LLM usando `context7-cli`.
- **`context7 keys rotate`** — nova operação de chave que avança manualmente o ponteiro da chave ativa para a próxima disponível no pool.
- **Flag curta `-q`** — alias curto para `--query` nos subcomandos `library` e `docs`.
- **Variável de ambiente `CONTEXT7_HOME`** — diretório base XDG alternativo para configuração (principalmente útil para testes e ambientes de CI).
- **Nova dependência** — `sys-locale 0.3` para auto-detect de idioma.
- **Novas dev-dependencies** — `assert_cmd 2` e `predicates 3` para testes de integração da CLI.
- **`rust-version = "1.75"`** declarado no Cargo.toml (MSRV).
- **Testes de integração** no diretório `tests/` usando `assert_cmd` e `predicates`.

### Alterado

- **Arquitetura modular** — refatorado de `context7.rs` monolítico (3006 linhas) para `src/` com 8 arquivos:
  - `src/lib.rs` — exports da API pública
  - `src/main.rs` — ponto de entrada do binário
  - `src/cli.rs` — parsing de argumentos com clap
  - `src/api.rs` — cliente HTTP e chamadas à API Context7
  - `src/storage.rs` — persistência de config XDG
  - `src/output.rs` — formatação colorida/JSON/texto
  - `src/errors.rs` — tipos de erro estruturados com `thiserror`
  - `src/i18n.rs` — tabela de lookup de mensagens bilíngues
- **Docstrings da API pública** — agora em inglês para corresponder às expectativas do público do docs.rs (comentários internos e mensagens para o usuário permanecem em português brasileiro).
- **README.md** — reescrita completa com estrutura bilíngue (inglês primeiro, depois Português). Adicionada seção "Por que usar?" com 8 benefícios, instalação rápida para 3 OS e links para os novos arquivos de docs.

### Corrigido

- **Falha de build no docs.rs** — build #3116184 da v0.1.0 falhou com `error: no library targets found in package`. Resolvido adicionando target `[lib]` no Cargo.toml.
- **Consolidação do `como_usa.md`** — conteúdo migrado e melhorado em `docs/HOW_TO_USE.md` com estrutura bilíngue e cobertura de 3 OS.

### Preservado

Todo comportamento CLI da v0.1.0 está preservado sem breaking changes:
- Subcomandos: `library`, `docs`, `keys`
- Aliases: `lib`, `search`, `doc`, `context`, `key`
- Layout de armazenamento XDG (Linux/macOS/Windows)
- Permissões do arquivo de chaves (`0o600` no Unix)
- Hierarquia de chaves de API (env var → XDG → `.env` → compile-time)
- Lógica de retry (3 tentativas, backoff 500ms → 1s → 2s)
- Flags `--json` e `--text`
- Todas as suboperações de `keys` da v0.1.0: `add`, `list`, `remove`, `clear`, `path`, `import`, `export` (mais a nova `rotate`, que foi posteriormente removida na v0.2.1 — a rotação é automática)

---

## [0.1.0] — 2026-04-08

### Adicionado

- Lançamento inicial publicado no [crates.io](https://crates.io/crates/context7-cli) e [GitHub](https://github.com/daniloaguiarbr/context7-cli).
- Binário Rust nativo `context7` — arquivo único, sem dependências de runtime.
- Subcomando `library` (aliases: `lib`, `search`) — busca bibliotecas Context7 por nome com contexto semântico opcional.
- Subcomando `docs` (aliases: `doc`, `context`) — busca documentação de biblioteca por ID com flags `--query`, `--text`, `--json`.
- Subcomando `keys` (alias: `key`) — gerencia chaves de API localmente via operações `add`, `list`, `remove`, `clear`, `path`, `import`, `export`.
- Armazenamento de config compatível com XDG (`~/.config/context7/config.toml` no Linux) com `chmod 600` no Unix.
- Rotação multi-chave com shuffle sem reposição.
- Retry com backoff exponencial: 3 tentativas, 500ms → 1s → 2s.
- Logging dual: stderr com cores ANSI + arquivo de estado XDG com rotação por deleção.
- Flag global `--json` para saída legível por máquina.
- `como_usa.md` — guia de uso em português brasileiro (791 linhas).
- Licença: MIT OR Apache-2.0.
