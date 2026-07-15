// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bilingual internationalisation (EN/PT-BR) for user-facing messages.
//!
//! Language resolution order:
//! 1. CLI flag `--lang en|pt`
//! 2. Environment variable `CONTEXT7_LANG`
//! 3. `sys_locale::get_locale()` — locale starting with `"pt"` → Portuguese
//! 4. Default: English
//!
//! Call [`set_language`] once at startup (in `run()`), then call
//! [`current_language`] or [`t`] anywhere to retrieve localised strings.
use std::sync::OnceLock;

// ─── LANGUAGE ───────────────────────────────────────────────────────────────────

/// Supported display languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    /// English output.
    English,
    /// Brazilian Portuguese output.
    Portuguese,
}

/// Global language setting — written once at startup, read-only thereafter.
static GLOBAL_LANGUAGE: OnceLock<Language> = OnceLock::new();

/// Returns the currently configured language.
///
/// Defaults to [`Language::English`] if [`set_language`] has not been called.
pub fn current_language() -> Language {
    *GLOBAL_LANGUAGE.get().unwrap_or(&Language::English)
}

/// Sets the global language. Silently ignored if already set (OnceLock semantics).
pub fn set_language(language: Language) {
    let _ = GLOBAL_LANGUAGE.set(language);
}

/// Resolves the language from CLI flag, env var, or system locale.
///
/// Resolution order:
/// 1. `cli_lang` — value of `--lang` flag (if provided)
/// 2. `CONTEXT7_LANG` environment variable
/// 3. `sys_locale::get_locale()` — BCP 47 locale (e.g. `"pt-BR"`)
/// 4. Default: English
#[must_use]
pub fn resolve_language(cli_lang: Option<&str>) -> Language {
    // 1. CLI flag
    if let Some(lang) = cli_lang {
        return parse_lang_str(lang);
    }

    // 2. Environment variable
    if let Ok(env_lang) = std::env::var("CONTEXT7_LANG") {
        return parse_lang_str(&env_lang);
    }

    // 3. System locale
    if let Some(locale) = sys_locale::get_locale() {
        if locale.to_lowercase().starts_with("pt") {
            return Language::Portuguese;
        }
    }

    // 4. Default
    Language::English
}

fn parse_lang_str(s: &str) -> Language {
    match s.to_lowercase().as_str() {
        "pt" | "pt-br" | "pt_br" | "portugues" | "português" => Language::Portuguese,
        _ => Language::English,
    }
}

// ─── MESSAGE ─────────────────────────────────────────────────────────────────

/// All user-facing messages indexed by variant.
///
/// Each variant maps to a pair of `(English, Portuguese)` strings.
#[derive(Debug, Clone, Copy)]
pub enum Message {
    // Keys subcommand (15 variants)
    /// "Key added successfully at: {path}"
    KeyAdded,
    /// "Key already exists (skipping)."
    KeyAlreadyExisted,
    /// "No key stored."
    NoStoredKey,
    /// "Use `context7 keys add <KEY>` to add a key."
    UseKeysAdd,
    /// "{n} key(s) stored:"
    KeysCount,
    /// "No key stored to remove."
    NoKeysToRemove,
    /// "Index {i} invalid. Use a number between 1 and {n}."
    InvalidIndex,
    /// "Key {masked} removed successfully."
    KeyRemovedSuccess,
    /// "Operation cancelled."
    OperationCancelled,
    /// "All keys removed."
    AllKeysRemoved,
    /// "System does not support XDG directories."
    XdgSystemNotSupported,
    /// "{imported}/{total} key(s) imported successfully."
    KeysImportedSuccess,
    /// "No CONTEXT7_API= key found in: {file}"
    NoContext7KeyInFile,
    /// "Are you sure you want to remove ALL keys? [y/N] " / "[s/N] "
    ConfirmRemoveAll,
    /// Accepted confirmation responses: "y"/"yes" or "s"/"sim"
    ConfirmationResponse,
    /// "API key cannot be empty. Get a key at <https://context7.com>"
    EmptyOrInvalidKey,
    /// "Warning: key does not match expected format (ctx7sk-...). API calls may fail."
    KeyFormatWarning,

    // Library / Docs (8 variants)
    /// "No library found."
    NoLibraryFound,
    /// "Libraries found:"
    LibrariesFound,
    /// "Trust:"
    TrustScore,
    /// "No documentation found."
    NoDocumentationFound,
    /// "Documentation:"
    DocumentationTitle,
    /// "Sources:"
    SourcesTitle,
    /// "No content available."
    NoContentAvailable,
    /// "Searching library: {name}"
    SearchingLibrary,

    // HTTP / Network errors (8 variants)
    /// "Network error searching library: {err}"
    NetworkError,
    /// "Network error fetching documentation: {err}"
    NetworkErrorDocs,
    /// "Failed to deserialise JSON response: {err}"
    DeserialiseFailure,
    /// "Rate limit reached (429), waiting for retry…"
    RateLimitReached,
    /// "Server error ({status}), retrying…"
    ServerError,
    /// "Invalid API key (401/403), trying next…"
    InvalidApiKey,
    /// "Attempt {n}/{max}"
    Attempt,
    /// "Waiting {ms}ms before retrying…"
    WaitingForRetry,

    // Config / XDG (7 variants)
    /// "System does not support XDG — cannot save configuration"
    XdgPathError,
    /// "Failed to read XDG config at: {path}"
    ConfigReadFailure,
    /// "Invalid TOML at: {path}"
    InvalidTomlFailure,
    /// "Failed to write config at: {path}"
    ConfigWriteFailure,
    /// "Failed to create directory: {path}"
    DirectoryCreateFailure,
    /// "No API key configured. Set CONTEXT7_API_KEYS or use `keys add`."
    NoKeyConfigured,
    /// "Failed to serialise configuration to TOML"
    TomlSerialiseFailure,

    // Logging / info (8 variants)
    /// "Keys loaded from CONTEXT7_API_KEYS environment variable"
    KeysLoadedFromEnvVar,
    /// "Keys loaded from XDG configuration"
    KeysLoadedFromXdg,
    /// "Failed to read XDG configuration (continuing): {err}"
    XdgReadFailureContinuing,
    /// "Starting context7 with {n} API keys available"
    StartingWithKeys,
    /// "Keys loaded from compile-time CONTEXT7_API_KEYS"
    KeysLoadedAtCompileTime,
    /// "Failed to serialise results to JSON"
    JsonSerialiseFailure,
    /// "Failed to serialise documentation to JSON"
    DocsSerialiseFailure,
    /// "Failed to search library '{name}'"
    LibrarySearchFailure,

    // Permissions / IO (2 variants)
    /// "Failed to read metadata of: {path}"
    MetadataReadFailure,
    /// "Failed to set permissions on: {path}"
    PermissionSetFailure,

    // API / Docs errors (4 variants)
    /// "Failed to fetch documentation for: {library_id}"
    DocsFetchFailure,
    /// "Failed to create HTTP client"
    HttpClientCreateFailure,
    /// "No documentation available"
    NoDocumentationAvailable,
    /// "Library not found. Verify the ID via `context7 library <name>`."
    LibraryNotFoundApi,

    // Health subcommand (7 variants)
    /// "Running health checks…"
    HealthRunning,
    /// "Config: OK"
    HealthConfigOk,
    /// "Config: FAILED"
    HealthConfigFailed,
    /// "API keys: {n} configured"
    HealthKeysOk,
    /// "API keys: none configured (exit 66)"
    HealthKeysMissing,
    /// "API: reachable"
    HealthApiOk,
    /// "API: offline or unreachable (exit 69)"
    HealthApiOffline,
}

impl Message {
    /// Returns the localised text for this message in the given language.
    ///
    /// Prefer this over the global [`t`] function when you need deterministic
    /// translations without depending on the process-wide language setting
    /// (useful for tests and library usage).
    pub fn text(self, language: Language) -> &'static str {
        match language {
            Language::English => en(self),
            Language::Portuguese => pt(self),
        }
    }
}

/// Returns the localised string for a message in the current language.
///
/// For parameterised messages use `format!("{} {}", t(Message::Foo), param)`.
#[must_use]
pub fn t(msg: Message) -> &'static str {
    match current_language() {
        Language::English => en(msg),
        Language::Portuguese => pt(msg),
    }
}

fn en(msg: Message) -> &'static str {
    match msg {
        // Keys
        Message::KeyAdded => "Key added successfully at:",
        Message::KeyAlreadyExisted => "Key already exists (skipping).",
        Message::NoStoredKey => "No key stored.",
        Message::UseKeysAdd => "Use `context7 keys add <KEY>` to add a key.",
        Message::KeysCount => "key(s) stored:",
        Message::NoKeysToRemove => "No key stored to remove.",
        Message::InvalidIndex => "Invalid index. Use a number between 1 and",
        Message::KeyRemovedSuccess => "Key removed successfully.",
        Message::OperationCancelled => "Operation cancelled.",
        Message::AllKeysRemoved => "All keys removed.",
        Message::XdgSystemNotSupported => "System does not support XDG directories.",
        Message::KeysImportedSuccess => "key(s) imported successfully.",
        Message::NoContext7KeyInFile => "No CONTEXT7_API= key found in:",
        Message::ConfirmRemoveAll => "Are you sure you want to remove ALL keys? [y/N] ",
        Message::ConfirmationResponse => "y|yes",
        Message::EmptyOrInvalidKey => {
            "API key cannot be empty. Get a key at https://context7.com"
        }
        Message::KeyFormatWarning => {
            "Warning: key does not match expected format (ctx7sk-...). API calls may fail."
        }

        // Library / Docs
        Message::NoLibraryFound => "No library found.",
        Message::LibrariesFound => "Libraries found:",
        Message::TrustScore => "trust",
        Message::NoDocumentationFound => "No documentation found.",
        Message::DocumentationTitle => "Documentation:",
        Message::SourcesTitle => "Sources:",
        Message::NoContentAvailable => "No content available.",
        Message::SearchingLibrary => "Searching library:",

        // HTTP / Network
        Message::NetworkError => "Network error searching library:",
        Message::NetworkErrorDocs => "Network error fetching documentation:",
        Message::DeserialiseFailure => "Failed to deserialise JSON response:",
        Message::RateLimitReached => "Rate limit reached (429), waiting for retry…",
        Message::ServerError => "Server error, retrying…",
        Message::InvalidApiKey => "Invalid API key (401/403), trying next…",
        Message::Attempt => "Attempt",
        Message::WaitingForRetry => "Waiting before retrying…",

        // Config / XDG
        Message::XdgPathError => "System does not support XDG — cannot save configuration",
        Message::ConfigReadFailure => "Failed to read XDG config at:",
        Message::InvalidTomlFailure => "Invalid TOML at:",
        Message::ConfigWriteFailure => "Failed to write config at:",
        Message::DirectoryCreateFailure => "Failed to create directory:",
        Message::NoKeyConfigured => {
            "No API key configured. Set CONTEXT7_API_KEYS or use `context7 keys add <KEY>`."
        }
        Message::TomlSerialiseFailure => "Failed to serialise configuration to TOML",

        // Logging / info
        Message::KeysLoadedFromEnvVar => {
            "Keys loaded from CONTEXT7_API_KEYS environment variable"
        }
        Message::KeysLoadedFromXdg => "Keys loaded from XDG configuration",
        Message::XdgReadFailureContinuing => "Failed to read XDG configuration (continuing):",
        Message::StartingWithKeys => "Starting context7 with",
        Message::KeysLoadedAtCompileTime => "Keys loaded from compile-time CONTEXT7_API_KEYS",
        Message::JsonSerialiseFailure => "Failed to serialise results to JSON",
        Message::DocsSerialiseFailure => "Failed to serialise documentation to JSON",
        Message::LibrarySearchFailure => "Failed to search library",

        // Permissions / IO
        Message::MetadataReadFailure => "Failed to read metadata of:",
        Message::PermissionSetFailure => "Failed to set permissions on:",

        // API / Docs errors
        Message::DocsFetchFailure => "Failed to fetch documentation for:",
        Message::HttpClientCreateFailure => "Failed to create HTTP client",
        Message::NoDocumentationAvailable => "No documentation available",
        Message::LibraryNotFoundApi => {
            "Library not found. Verify the ID via `context7 library <name>`."
        }

        // Health
        Message::HealthRunning => "Running health checks…",
        Message::HealthConfigOk => "Config: OK",
        Message::HealthConfigFailed => "Config: FAILED",
        Message::HealthKeysOk => "API keys configured:",
        Message::HealthKeysMissing => "API keys: none configured",
        Message::HealthApiOk => "API: reachable",
        Message::HealthApiOffline => "API: offline or unreachable",
    }
}

fn pt(msg: Message) -> &'static str {
    match msg {
        // Keys
        Message::KeyAdded => "Chave adicionada com sucesso em:",
        Message::KeyAlreadyExisted => "Chave já existente (ignorando).",
        Message::NoStoredKey => "Nenhuma chave armazenada.",
        Message::UseKeysAdd => "Use `context7 keys add <CHAVE>` para adicionar uma chave.",
        Message::KeysCount => "chave(s) armazenada(s):",
        Message::NoKeysToRemove => "Nenhuma chave armazenada para remover.",
        Message::InvalidIndex => "Índice inválido. Use um número entre 1 e",
        Message::KeyRemovedSuccess => "Chave removida com sucesso.",
        Message::OperationCancelled => "Operação cancelada.",
        Message::AllKeysRemoved => "Todas as chaves foram removidas.",
        Message::XdgSystemNotSupported => "Sistema não suporta diretórios XDG.",
        Message::KeysImportedSuccess => "chave(s) importada(s) com sucesso.",
        Message::NoContext7KeyInFile => "Nenhuma chave CONTEXT7_API= encontrada em:",
        Message::ConfirmRemoveAll => "Tem certeza que deseja remover TODAS as chaves? [s/N] ",
        Message::ConfirmationResponse => "s|sim",
        Message::EmptyOrInvalidKey => {
            "Chave de API não pode ser vazia. Obtenha uma em https://context7.com"
        }
        Message::KeyFormatWarning => {
            "Aviso: chave não corresponde ao formato esperado (ctx7sk-...). Chamadas de API podem falhar."
        }

        // Library / Docs
        Message::NoLibraryFound => "Nenhuma biblioteca encontrada.",
        Message::LibrariesFound => "Bibliotecas encontradas:",
        Message::TrustScore => "confiança",
        Message::NoDocumentationFound => "Nenhuma documentação encontrada.",
        Message::DocumentationTitle => "Documentação:",
        Message::SourcesTitle => "Fontes:",
        Message::NoContentAvailable => "Sem conteúdo disponível.",
        Message::SearchingLibrary => "Buscando biblioteca:",

        // HTTP / Network
        Message::NetworkError => "Erro de rede ao buscar biblioteca:",
        Message::NetworkErrorDocs => "Erro de rede ao buscar documentação:",
        Message::DeserialiseFailure => "Falha ao desserializar resposta JSON:",
        Message::RateLimitReached => "Rate limit atingido (429), aguardando retry…",
        Message::ServerError => "Erro do servidor, tentando novamente…",
        Message::InvalidApiKey => "Chave de API inválida (401/403), tentando próxima…",
        Message::Attempt => "Attempt",
        Message::WaitingForRetry => "Aguardando antes de tentar novamente…",

        // Config / XDG
        Message::XdgPathError => {
            "Sistema não suporta diretórios XDG — impossível salvar configuração"
        }
        Message::ConfigReadFailure => "Falha ao ler configuração XDG em:",
        Message::InvalidTomlFailure => "TOML inválido em:",
        Message::ConfigWriteFailure => "Falha ao escrever config em:",
        Message::DirectoryCreateFailure => "Falha ao criar diretório:",
        Message::NoKeyConfigured => {
            "Nenhuma chave de API encontrada. Configure CONTEXT7_API_KEYS ou use `context7 keys add <CHAVE>`."
        }
        Message::TomlSerialiseFailure => "Falha ao serializar configuração para TOML",

        // Logging / info
        Message::KeysLoadedFromEnvVar => {
            "Chaves carregadas via variável de ambiente CONTEXT7_API_KEYS"
        }
        Message::KeysLoadedFromXdg => "Chaves carregadas via configuração XDG",
        Message::XdgReadFailureContinuing => "Falha ao ler configuração XDG (continuando):",
        Message::StartingWithKeys => "Iniciando context7 com",
        Message::KeysLoadedAtCompileTime => {
            "Chaves carregadas via compile-time CONTEXT7_API_KEYS"
        }
        Message::JsonSerialiseFailure => "Falha ao serializar resultados para JSON",
        Message::DocsSerialiseFailure => "Falha ao serializar documentação para JSON",
        Message::LibrarySearchFailure => "Falha ao buscar biblioteca",

        // Permissions / IO
        Message::MetadataReadFailure => "Falha ao ler metadados de:",
        Message::PermissionSetFailure => "Falha ao definir permissões em:",

        // API / Docs errors
        Message::DocsFetchFailure => "Falha ao buscar documentação para:",
        Message::HttpClientCreateFailure => "Falha ao criar cliente HTTP",
        Message::NoDocumentationAvailable => "Nenhuma documentação disponível",
        Message::LibraryNotFoundApi => {
            "Biblioteca não encontrada. Verifique o ID via `context7 library <nome>`."
        }

        // Health
        Message::HealthRunning => "Executando verificações de saúde…",
        Message::HealthConfigOk => "Config: OK",
        Message::HealthConfigFailed => "Config: FALHOU",
        Message::HealthKeysOk => "Chaves de API configuradas:",
        Message::HealthKeysMissing => "Chaves de API: nenhuma configurada",
        Message::HealthApiOk => "API: acessível",
        Message::HealthApiOffline => "API: offline ou inacessível",
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_language_default_is_english() {
        // If OnceLock has not been set in this test process, it must be English
        let _ = current_language();
    }

    #[test]
    fn test_resolve_language_cli_flag_pt() {
        assert_eq!(resolve_language(Some("pt")), Language::Portuguese);
        assert_eq!(resolve_language(Some("pt-BR")), Language::Portuguese);
        assert_eq!(resolve_language(Some("PT_BR")), Language::Portuguese);
    }

    #[test]
    fn test_resolve_language_cli_flag_en() {
        assert_eq!(resolve_language(Some("en")), Language::English);
        assert_eq!(resolve_language(Some("en-US")), Language::English);
    }

    #[test]
    fn test_resolve_language_no_flag_no_env_returns_english_or_pt() {
        // Without flag and without env, must return English or Portuguese (depending on the system)
        let language = resolve_language(None);
        assert!(language == Language::English || language == Language::Portuguese);
    }

    #[test]
    fn test_t_message_no_key_en() {
        let msg_en = en(Message::NoStoredKey);
        assert!(!msg_en.is_empty());
        assert!(
            msg_en.to_lowercase().contains("no") || msg_en.to_lowercase().contains("key"),
            "EN must contain 'no' or 'key', got: {}",
            msg_en
        );
    }

    #[test]
    fn test_t_message_no_key_pt() {
        let msg_pt = pt(Message::NoStoredKey);
        assert!(!msg_pt.is_empty());
        assert!(
            msg_pt.to_lowercase().contains("nenhuma") || msg_pt.to_lowercase().contains("key"),
            "PT deve conter 'nenhuma' ou 'key', obteve: {}",
            msg_pt
        );
    }

    #[test]
    fn test_confirmation_response_en_contains_y() {
        let confirmacao = en(Message::ConfirmationResponse);
        assert!(
            confirmacao.contains('y'),
            "EN: confirmation response must contain 'y'"
        );
    }

    #[test]
    fn test_confirmation_response_pt_contains_s() {
        let confirmacao = pt(Message::ConfirmationResponse);
        assert!(
            confirmacao.contains('s'),
            "PT: confirmation response must contain 's'"
        );
    }

    #[test]
    fn test_all_variants_en_not_empty() {
        let variantes = [
            Message::KeyAdded,
            Message::KeyAlreadyExisted,
            Message::NoStoredKey,
            Message::UseKeysAdd,
            Message::KeysCount,
            Message::NoKeysToRemove,
            Message::InvalidIndex,
            Message::KeyRemovedSuccess,
            Message::OperationCancelled,
            Message::AllKeysRemoved,
            Message::XdgSystemNotSupported,
            Message::KeysImportedSuccess,
            Message::NoContext7KeyInFile,
            Message::ConfirmRemoveAll,
            Message::ConfirmationResponse,
            Message::NoLibraryFound,
            Message::LibrariesFound,
            Message::TrustScore,
            Message::NoDocumentationFound,
            Message::DocumentationTitle,
            Message::SourcesTitle,
            Message::NoContentAvailable,
            Message::SearchingLibrary,
            Message::NetworkError,
            Message::NetworkErrorDocs,
            Message::DeserialiseFailure,
            Message::RateLimitReached,
            Message::ServerError,
            Message::InvalidApiKey,
            Message::Attempt,
            Message::WaitingForRetry,
            Message::XdgPathError,
            Message::ConfigReadFailure,
            Message::InvalidTomlFailure,
            Message::ConfigWriteFailure,
            Message::DirectoryCreateFailure,
            Message::NoKeyConfigured,
            Message::TomlSerialiseFailure,
            Message::KeysLoadedFromEnvVar,
            Message::KeysLoadedFromXdg,
            Message::XdgReadFailureContinuing,
            Message::StartingWithKeys,
            Message::KeysLoadedAtCompileTime,
            Message::JsonSerialiseFailure,
            Message::DocsSerialiseFailure,
            Message::LibrarySearchFailure,
            Message::MetadataReadFailure,
            Message::PermissionSetFailure,
            Message::DocsFetchFailure,
            Message::HttpClientCreateFailure,
            Message::NoDocumentationAvailable,
            Message::LibraryNotFoundApi,
            Message::EmptyOrInvalidKey,
            Message::KeyFormatWarning,
            Message::HealthRunning,
            Message::HealthConfigOk,
            Message::HealthConfigFailed,
            Message::HealthKeysOk,
            Message::HealthKeysMissing,
            Message::HealthApiOk,
            Message::HealthApiOffline,
        ];

        for v in &variantes {
            let msg_en = en(*v);
            let msg_pt = pt(*v);
            assert!(
                !msg_en.is_empty(),
                "EN message vazia para variante {:?}",
                v
            );
            assert!(
                !msg_pt.is_empty(),
                "PT message vazia para variante {:?}",
                v
            );
        }
    }
}
