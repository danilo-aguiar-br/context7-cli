# Pull Request

## Description / Descrição

<!-- EN: Briefly describe the changes and motivation. -->
<!-- PT: Descreva brevemente as mudanças e a motivação. -->

## Type of Change / Tipo de Mudança

<!-- EN: Mark the applicable option with an "x". -->
<!-- PT: Marque a opção aplicável com um "x". -->

- [ ] Bug fix / Correção de bug (`fix:`)
- [ ] New feature / Nova funcionalidade (`feat:`)
- [ ] Breaking change / Mudança incompatível (`feat!:` / `fix!:`)
- [ ] Documentation / Documentação (`docs:`)
- [ ] Refactoring / Refatoração (`refactor:`)
- [ ] Performance improvement / Melhoria de desempenho (`perf:`)
- [ ] Tests / Testes (`test:`)
- [ ] CI / Infraestrutura (`ci:`)

## Checklist / Lista de Verificação

### Code Quality / Qualidade de Código

- [ ] `cargo fmt --check` passes / passa
- [ ] `cargo clippy -- -D warnings` passes with zero warnings / passa sem warnings
- [ ] `cargo check` passes / passa
- [ ] `cargo doc --no-deps` passes without warnings / passa sem warnings

### Tests / Testes

- [ ] `cargo test` — all tests pass / todos os testes passam
- [ ] New tests added for new functionality / Novos testes adicionados para nova funcionalidade
- [ ] No `unwrap()` or `expect()` in production code / Sem `unwrap()` ou `expect()` em código de produção
- [ ] No `println!` debug in production code / Sem `println!` de debug em código de produção

### Cross-Platform / Multiplataforma

- [ ] Tested on Linux / Testado no Linux
- [ ] Tested on macOS / Testado no macOS (or CI passes / ou CI passa)
- [ ] Tested on Windows / Testado no Windows (or CI passes / ou CI passa)

### Documentation / Documentação

- [ ] `README.md` updated if applicable / `README.md` atualizado se aplicável
- [ ] `CHANGELOG.md` updated / `CHANGELOG.md` atualizado
- [ ] Bilingual (EN + PT) if touching docs / Bilíngue (EN + PT) se tocar docs

## Related Issues / Issues Relacionadas

<!-- EN: Closes #xxx / PT: Fecha #xxx -->

## Additional Notes / Notas Adicionais

<!-- EN: Any additional context or screenshots. -->
<!-- PT: Qualquer contexto adicional ou screenshots. -->

## Bot PRs / PRs automatizados

When merging PRs from automation bots, use **Squash and merge**
with "Use pull request title and description". Do NOT preserve
`Co-authored-by: <bot>` trailers on main branch.

Ao mergear PRs de bots de automação, use **Squash and merge**
com "Use pull request title and description". NÃO preserve trailers
`Co-authored-by: <bot>` na branch main.
