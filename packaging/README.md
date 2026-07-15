# Pacotes de Distribuição / Distribution Packages

Este diretório contém os manifestos para distribuição do `context7-cli` em formatos
alternativos além do `cargo install`.

This directory contains manifests for distributing `context7-cli` in formats
beyond `cargo install`.

---

## Flatpak

O Flatpak distribui o context7-cli como uma aplicação sandboxed com acesso controlado
à rede. Requer o SDK Freedesktop com extensão Rust estável.

Flatpak distributes context7-cli as a sandboxed application with controlled network
access. Requires the Freedesktop SDK with stable Rust extension.

### Build local / Local build

```bash
# Instalar o flatpak-builder se ainda não tiver
# Install flatpak-builder if you don't have it
sudo apt install flatpak-builder  # Debian/Ubuntu
brew install flatpak-builder      # macOS (via Homebrew)

# Adicionar o repositório Flathub
# Add the Flathub repository
flatpak remote-add --if-not-exists flathub https://flathub.org/repo/flathub.flatpakrepo

# Instalar o SDK e extensão Rust
# Install the SDK and Rust extension
flatpak install flathub org.freedesktop.Platform//24.08 org.freedesktop.Sdk//24.08
flatpak install flathub org.freedesktop.Sdk.Extension.rust-stable//24.08

# Preparar dependências offline (vendor/)
# Prepare offline dependencies (vendor/)
cargo vendor
tar -czf packaging/flatpak/vendor.tar.gz vendor/

# Buildar o Flatpak
# Build the Flatpak
flatpak-builder --force-clean build-dir \
  packaging/flatpak/br.com.daniloaguiar.context7-cli.yaml

# Instalar localmente para teste
# Install locally for testing
flatpak-builder --user --install --force-clean build-dir \
  packaging/flatpak/br.com.daniloaguiar.context7-cli.yaml

# Executar
# Run
flatpak run br.com.daniloaguiar.context7-cli search react
```

---

## Snap

O Snap distribui o context7-cli com confinamento estrito (`strict`), usando apenas
as interfaces `network` e `home`.

Snap distributes context7-cli with strict confinement, using only the `network`
and `home` interfaces.

### Build local / Local build

```bash
# Instalar o snapcraft se ainda não tiver
# Install snapcraft if you don't have it
sudo snap install snapcraft --classic

# Buildar o Snap a partir do diretório raiz do projeto
# Build the Snap from the project root directory
cd packaging/snap
snapcraft

# Instalar localmente para teste (sem assinatura)
# Install locally for testing (unsigned)
sudo snap install context7-cli_*.snap --dangerous

# Executar
# Run
context7-cli.context7 search react

# Alias opcional: tornar o comando simplesmente 'context7'
# Optional alias: make the command simply 'context7'
sudo snap alias context7-cli.context7 context7
```

---

## Universal Binary macOS

O job `macos-universal` do GitHub Actions combina os binários `aarch64-apple-darwin`
e `x86_64-apple-darwin` em um único Universal Binary usando `lipo`.

The `macos-universal` GitHub Actions job combines `aarch64-apple-darwin` and
`x86_64-apple-darwin` binaries into a single Universal Binary using `lipo`.

### Verificar o Universal Binary / Verify the Universal Binary

```bash
# Ver as arquiteturas contidas no universal binary
# See the architectures contained in the universal binary
lipo -info context7-universal

# Saída esperada / Expected output:
# Architectures in the fat file: context7-universal are: x86_64 arm64

# Extrair apenas a arquitetura arm64
# Extract only the arm64 architecture
lipo -extract arm64 context7-universal -output context7-arm64

# Extrair apenas a arquitetura x86_64
# Extract only the x86_64 architecture
lipo -extract x86_64 context7-universal -output context7-x86_64

# Verificar assinatura de código
# Verify code signature
codesign -dv --verbose=4 context7-universal
```

### Build local do Universal Binary / Local Universal Binary build

```bash
# Adicionar os targets macOS se ainda não tiver
# Add macOS targets if you don't have them
rustup target add aarch64-apple-darwin x86_64-apple-darwin

# Compilar para ambas as arquiteturas
# Compile for both architectures
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Combinar com lipo
# Combine with lipo
lipo -create \
  target/aarch64-apple-darwin/release/context7 \
  target/x86_64-apple-darwin/release/context7 \
  -output context7-universal

# Assinar com adhoc (sem certificado Apple Developer)
# Sign with adhoc (without Apple Developer certificate)
codesign --sign - --force context7-universal

# Verificar resultado
# Verify result
lipo -info context7-universal
```

### Notarização Apple (produção) / Apple Notarization (production)

Para distribuição fora da Mac App Store com notarização, configure os secrets
no repositório GitHub:

For distribution outside the Mac App Store with notarization, configure the
secrets in the GitHub repository:

| Secret | Descrição / Description |
|---|---|
| `APPLE_TEAM_ID` | Team ID da conta Apple Developer (ex: `ABCD1234EF`) |
| `APPLE_ID` | Apple ID (email) da conta Developer |
| `APPLE_APP_PASSWORD` | App-specific password gerada em appleid.apple.com |

Se `APPLE_TEAM_ID` estiver vazio ou ausente, o workflow aplica assinatura adhoc
automaticamente (funcional mas mostra aviso no macOS).

If `APPLE_TEAM_ID` is empty or absent, the workflow applies adhoc signing
automatically (functional but shows a warning on macOS).
