# Security Policy / Política de Segurança

## Supported Versions / Versões Suportadas

| Version / Versão | Supported / Suportada |
| ---------------- | --------------------- |
| Latest (crates.io) | ✅ Yes / Sim |
| Older releases | ❌ No / Não |

Only the latest published version on [crates.io](https://crates.io/crates/context7-cli) receives security fixes. Users are encouraged to keep `context7-cli` up to date via `cargo install context7-cli --force`.

---

## Reporting a Vulnerability / Reportando uma Vulnerabilidade

### EN — Reporting

**Please do not report security vulnerabilities through public GitHub issues.**

**Primary channel (preferred):** GitHub Security Advisories  
→ [https://github.com/danilo-aguiar-br/context7-cli/security/advisories/new](https://github.com/danilo-aguiar-br/context7-cli/security/advisories/new)

**Alternate channel:** Email — daniloaguiarbr@proton.me

#### Response timeline

| Step | Timeline |
| ---- | -------- |
| Acknowledgment of receipt | Within 48 hours |
| Status update | Within 7 days |
| Fix or mitigation | Within 30 days (severity-dependent) |

#### What to include in your report

- A clear description of the vulnerability
- Step-by-step instructions to reproduce it
- Potential impact and attack vector
- Affected version(s)
- Any suggested fix (optional, but appreciated)

We will credit you in the release notes unless you prefer to remain anonymous.

---

### PT — Reportando

**Por favor, não reporte vulnerabilidades de segurança através de issues públicas no GitHub.**

**Canal primário (preferido):** GitHub Security Advisories  
→ [https://github.com/danilo-aguiar-br/context7-cli/security/advisories/new](https://github.com/danilo-aguiar-br/context7-cli/security/advisories/new)

**Canal alternativo:** E-mail — daniloaguiarbr@proton.me

#### Cronograma de resposta

| Etapa | Prazo |
| ----- | ----- |
| Confirmação de recebimento | Em até 48 horas |
| Atualização de status | Em até 7 dias |
| Correção ou mitigação | Em até 30 dias (dependendo da gravidade) |

#### O que incluir no relatório

- Descrição clara da vulnerabilidade
- Instruções passo a passo para reproduzi-la
- Impacto potencial e vetor de ataque
- Versão(ões) afetada(s)
- Sugestão de correção (opcional, mas bem-vinda)

Você será creditado nas notas de versão, a menos que prefira permanecer anônimo.

---

## Security Measures / Medidas de Segurança

### EN

This project applies the following security practices:

- **`cargo audit`** — dependencies are checked against the [RustSec Advisory Database](https://rustsec.org/) on every CI run.
- **`cargo deny`** — license compatibility, supply-chain integrity, and banned crates are enforced via `deny.toml`.
- **No GitHub Actions / automated dependency bots** — CI/CD automation was removed from this repository as part of the ownership migration; dependency review is done manually before releases.
- **API key storage** — keys are stored locally in the XDG config directory (`~/.config/context7/` or `$CONTEXT7_HOME`). They are never logged, printed to stdout, or transmitted anywhere other than the official Context7 API endpoint over HTTPS.
- **No secrets in source** — the repository contains no hardcoded credentials, tokens, or private keys.
- **Zeroize memory** — API keys use the `zeroize` crate with `#[derive(ZeroizeOnDrop)]`. Memory holding key material is automatically zeroed when the value is dropped, preventing extraction from memory dumps or core files.
- **ChaveApi newtype** — the `ChaveApi` newtype masks keys in both `Debug` and `Display` trait implementations. Keys never leak to logs, stack traces, or error messages.
- **Windows reserved filename validation** — `CONTEXT7_HOME` values are validated against Windows reserved filenames (`CON`, `PRN`, `NUL`, `AUX`, `COM1`..`COM9`, `LPT1`..`LPT9`) to prevent path injection attacks on Windows systems.
- **Unicode NFC normalization** — paths are normalized to Unicode NFC form to prevent path confusion attacks on macOS HFS+ and other filesystems where different Unicode representations can resolve to the same path.

### PT

Este projeto aplica as seguintes práticas de segurança:

- **`cargo audit`** — as dependências são verificadas contra o [RustSec Advisory Database](https://rustsec.org/) em cada execução de CI.
- **`cargo deny`** — compatibilidade de licenças, integridade da cadeia de suprimentos e crates banidas são verificadas via `deny.toml`.
- **Sem GitHub Actions / bots automáticos de dependências** — a automação de CI/CD foi removida deste repositório na migração de ownership; revisão de dependências é feita manualmente antes de releases.
- **Armazenamento de chaves de API** — as chaves são armazenadas localmente no diretório de configuração XDG (`~/.config/context7/` ou `$CONTEXT7_HOME`). Elas nunca são registradas em logs, impressas em stdout ou transmitidas para qualquer lugar além do endpoint oficial da Context7 API via HTTPS.
- **Sem segredos no código-fonte** — o repositório não contém credenciais, tokens ou chaves privadas embutidas no código.
- **Zeroize de memória** — as chaves de API usam o crate `zeroize` com `#[derive(ZeroizeOnDrop)]`. A memória contendo o material da chave é automaticamente zerada quando o valor é dropado, prevenindo extração a partir de dumps de memória ou core files.
- **Newtype ChaveApi** — o newtype `ChaveApi` mascara chaves nas implementações dos traits `Debug` e `Display`. Chaves nunca vazam para logs, stack traces ou mensagens de erro.
- **Validação de nomes reservados do Windows** — valores de `CONTEXT7_HOME` são validados contra nomes de arquivo reservados do Windows (`CON`, `PRN`, `NUL`, `AUX`, `COM1`..`COM9`, `LPT1`..`LPT9`) para prevenir ataques de injeção de caminho em sistemas Windows.
- **Normalização Unicode NFC** — caminhos são normalizados para a forma Unicode NFC para prevenir ataques de confusão de caminho no macOS HFS+ e outros sistemas de arquivos onde representações Unicode diferentes podem resolver para o mesmo caminho.

---

## License / Licença

This project is licensed under either of **MIT** or **Apache-2.0** at your option.  
Este projeto é licenciado sob **MIT** ou **Apache-2.0** à sua escolha.
