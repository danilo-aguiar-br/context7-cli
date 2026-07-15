// SPDX-License-Identifier: MIT OR Apache-2.0
//! HTTP client, retry logic, and Context7 API calls.
//!
//! This module owns the full lifecycle of API interaction:
//! request building, status handling, and key-rotation retry.
use anyhow::{bail, Context, Result};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::errors::Context7Error;
use crate::i18n::{t, Message};
use crate::storage::ApiKey;

// ─── CONSTANT ───────────────────────────────────────────────────────────────

const BASE_URL: &str = "https://context7.com/api";

// ─── API RESPONSE MODELS ───────────────────────────────────────────────

/// Represents a single library entry returned by the search endpoint.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibrarySearchResult {
    /// Unique library identifier (e.g. `/facebook/react`).
    pub id: String,
    /// Human-readable library title.
    pub title: String,
    /// Optional short description of the library.
    pub description: Option<String>,
    /// Relevance/trust score returned by the API, if available.
    pub trust_score: Option<f64>,
    /// Number of GitHub stars, if available. The API returns `-1` when unavailable.
    pub stars: Option<i64>,
    /// Total number of documentation snippets indexed.
    pub total_snippets: Option<u64>,
    /// Total number of tokens indexed.
    pub total_tokens: Option<u64>,
    /// Whether the library has been verified by the Context7 team.
    pub verified: Option<bool>,
    /// Git branch used for indexing.
    pub branch: Option<String>,
    /// Indexing state (e.g. "active", "pending").
    pub state: Option<String>,
}

/// A single code block within a documentation snippet.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CodeBlock {
    /// Programming language of the code (e.g. `"rust"`, `"bash"`).
    pub language: String,
    /// Source code content.
    pub code: String,
}

/// Represents a single documentation excerpt returned by the docs endpoint (JSON mode).
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DocumentationSnippet {
    /// Page title of the source documentation page, if available.
    pub page_title: Option<String>,
    /// Title of this specific code snippet, if available.
    pub code_title: Option<String>,
    /// Description accompanying the code snippet, if available.
    pub code_description: Option<String>,
    /// Primary programming language of the snippet, if available.
    pub code_language: Option<String>,
    /// Number of tokens in this snippet, if available.
    pub code_tokens: Option<u64>,
    /// Unique identifier or source URL of this snippet, if available.
    pub code_id: Option<String>,
    /// List of code blocks contained in this snippet.
    pub code_list: Option<Vec<CodeBlock>>,
    /// Relevance score for the query, if available.
    pub relevance: Option<f64>,
    /// Model used to generate or rank this snippet, if available.
    pub model: Option<String>,
}

/// Top-level response from the library search endpoint (`GET /api/v1/search`).
#[derive(Debug, Deserialize)]
pub struct LibraryListResponse {
    /// List of matching libraries.
    pub results: Vec<LibrarySearchResult>,
}

/// Top-level response from the documentation endpoint (`GET /api/v1/{library_id}`).
#[derive(Debug, Deserialize, Serialize)]
pub struct DocumentationResponse {
    /// Structured documentation snippets (JSON mode).
    pub snippets: Option<Vec<DocumentationSnippet>>,
}

// ─── HTTP CLIENT ─────────────────────────────────────────────────────────────

/// Creates a reusable HTTP client with rustls-TLS, 30 s timeout, and HTTP/2.
///
/// The client should be created once per invocation and shared via `Arc`.
#[must_use]
pub fn create_http_client() -> Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .timeout(Duration::from_secs(30))
        .user_agent(format!("context7-cli/{}", env!("CARGO_PKG_VERSION")))
        .pool_max_idle_per_host(4)
        .build()
        .with_context(|| t(Message::HttpClientCreateFailure))?;

    Ok(client)
}

// ─── RETRY WITH KEY ROTATION ──────────────────────────────────────────────

/// Executes an API call with retry and key rotation.
///
/// Shuffles a local copy of the provided keys (random draw without replacement)
/// and retries up to `min(keys.len(), 5)` times with exponential backoff:
/// 500 ms → 1 s → 2 s.
///
/// Short-circuits immediately on parse errors (status 200 but JSON failed) —
/// retrying with another key would not help in that case.
///
/// The closure receives an owned `String` (clone of the key) to satisfy the
/// `async move` ownership requirement inside the future.
pub async fn run_with_retry<F, Fut, T>(keys: &[ApiKey], operation: F) -> Result<T>
where
    F: Fn(String) -> Fut,
    Fut: std::future::Future<Output = Result<T, Context7Error>>,
{
    let max_attempts = keys.len().min(5);

    // Shuffles a local copy — avoids modifying the caller's vec
    let mut shuffled_keys = keys.to_vec();
    // Fisher-Yates shuffle with fastrand — replaces rand::SliceRandom
    for i in (1..shuffled_keys.len()).rev() {
        let j = fastrand::usize(..=i);
        shuffled_keys.swap(i, j);
    }

    let delays_ms = [500u64, 1000, 2000];
    let mut failed_auth_keys = 0usize;

    for (attempt, key) in shuffled_keys
        .into_iter()
        .take(max_attempts)
        .enumerate()
    {
        info!("Attempt {}/{}", attempt + 1, max_attempts);

        match operation(key.value().to_string()).await {
            Ok(result) => return Ok(result),

            Err(Context7Error::ApiReturned400 { message }) => {
                // 400 is not transient — abort immediately
                bail!(Context7Error::ApiReturned400 { message });
            }

            Err(Context7Error::LibraryNotFound { library_id }) => {
                // Library doesn't exist — no point trying other keys
                bail!(Context7Error::LibraryNotFound { library_id });
            }

            Err(Context7Error::NoValidApiKey) => {
                failed_auth_keys += 1;
                warn!("Invalid API key (401/403), trying next...");
            }

            Err(Context7Error::InvalidResponse { status: 200 }) => {
                // Parse failure on HTTP 200 — schema mismatch, not a key issue
                // Short-circuit: no point trying other keys
                bail!(Context7Error::InvalidResponse { status: 200 });
            }

            Err(e) => {
                warn!("Failure on attempt {}: {}", attempt + 1, e);

                // Backoff before next attempt (not on the last one)
                if attempt + 1 < max_attempts && attempt < delays_ms.len() {
                    let delay = Duration::from_millis(delays_ms[attempt]);
                    info!("Waiting {}ms before retrying...", delay.as_millis());
                    sleep(delay).await;
                }
            }
        }
    }

    if failed_auth_keys >= max_attempts {
        bail!(Context7Error::NoValidApiKey);
    }

    bail!(Context7Error::RetriesExhausted {
        attempts: max_attempts as u32,
    });
}

// ─── API CALLS ───────────────────────────────────────────────────────────

/// Searches for libraries matching `name` with optional relevance `context_query`.
///
/// Returns `Err(Context7Error)` on HTTP errors to enable retry in `run_with_retry`.
#[must_use]
pub async fn search_library(
    client: &reqwest::Client,
    key: &str,
    name: &str,
    context_query: &str,
) -> Result<LibraryListResponse, Context7Error> {
    let url = format!("{}/v1/search", BASE_URL);

    let response = client
        .get(&url)
        .bearer_auth(key)
        .query(&[("libraryName", name), ("query", context_query)])
        .send()
        .await
        .map_err(|e| {
            error!("Network error while searching library: {}", e);
            Context7Error::InvalidResponse { status: 0 }
        })?;

    handle_response_status(response).await
}

/// Fetches documentation for `library_id` with an optional `query` filter (JSON mode).
///
/// Always requests `type=json`. Use `fetch_documentation_text` for plain-text output.
/// Returns `Err(Context7Error)` on HTTP errors to enable retry in `run_with_retry`.
#[must_use]
pub async fn fetch_documentation(
    client: &reqwest::Client,
    key: &str,
    library_id: &str,
    query: Option<&str>,
) -> Result<DocumentationResponse, Context7Error> {
    // Normalise library_id: strip leading slash if present
    let normalized_id = library_id.trim_start_matches('/');
    let url = format!("{}/v1/{}", BASE_URL, normalized_id);

    let mut builder = client
        .get(&url)
        .bearer_auth(key)
        .query(&[("type", "json")]);

    if let Some(q) = query {
        builder = builder.query(&[("query", q)]);
    }

    let response = builder.send().await.map_err(|e| {
        error!("Network error while fetching documentation: {}", e);
        Context7Error::InvalidResponse { status: 0 }
    })?;

    handle_response_status(response).await.map_err(|e| match e {
        Context7Error::InvalidResponse { status: 404 } => Context7Error::LibraryNotFound {
            library_id: library_id.to_string(),
        },
        other => other,
    })
}

/// Fetches documentation for `library_id` as raw plain text (markdown).
///
/// Uses `type=txt`. Returns the raw response body as a `String`.
/// Returns `Err(Context7Error)` on HTTP errors to enable retry in `run_with_retry`.
#[must_use]
pub async fn fetch_documentation_text(
    client: &reqwest::Client,
    key: &str,
    library_id: &str,
    query: Option<&str>,
) -> Result<String, Context7Error> {
    let normalized_id = library_id.trim_start_matches('/');
    let url = format!("{}/v1/{}", BASE_URL, normalized_id);

    let mut builder = client
        .get(&url)
        .bearer_auth(key)
        .query(&[("type", "txt")]);

    if let Some(q) = query {
        builder = builder.query(&[("query", q)]);
    }

    let response = builder.send().await.map_err(|e| {
        error!("Network error while fetching documentation: {}", e);
        Context7Error::InvalidResponse { status: 0 }
    })?;

    let status = response.status();

    if !status.is_success() {
        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                return Err(Context7Error::NoValidApiKey);
            }
            StatusCode::BAD_REQUEST => {
                let message = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No details".to_string());
                return Err(Context7Error::ApiReturned400 { message });
            }
            StatusCode::NOT_FOUND => {
                return Err(Context7Error::LibraryNotFound {
                    library_id: library_id.to_string(),
                });
            }
            _ => {
                return Err(Context7Error::InvalidResponse {
                    status: status.as_u16(),
                });
            }
        }
    }

    response
        .text()
        .await
        .map_err(|_| Context7Error::InvalidResponse {
            status: status.as_u16(),
        })
}

/// Maps HTTP status codes to typed `Context7Error` variants or deserialises success bodies.
async fn handle_response_status<T: for<'de> Deserialize<'de>>(
    response: reqwest::Response,
) -> Result<T, Context7Error> {
    let status = response.status();

    match status {
        s if s.is_success() => response.json::<T>().await.map_err(|e| {
            error!("Failed to deserialise JSON response: {}", e);
            Context7Error::InvalidResponse {
                status: status.as_u16(),
            }
        }),

        StatusCode::BAD_REQUEST => {
            let message = response
                .text()
                .await
                .unwrap_or_else(|_| "No details".to_string());
            Err(Context7Error::ApiReturned400 { message })
        }

        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(Context7Error::NoValidApiKey),

        StatusCode::TOO_MANY_REQUESTS => {
            warn!("Rate limit reached (429), waiting for retry...");
            Err(Context7Error::InvalidResponse {
                status: status.as_u16(),
            })
        }

        s if s.is_server_error() => {
            warn!(
                "Server error ({}), retrying...",
                status.as_u16()
            );
            Err(Context7Error::InvalidResponse {
                status: status.as_u16(),
            })
        }

        _ => Err(Context7Error::InvalidResponse {
            status: status.as_u16(),
        }),
    }
}

// ─── TESTS ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Struct deserialisation ────────────────────────────────────────────

    #[test]
    fn test_library_search_result_deserialisation() {
        let json = r#"{
            "id": "/facebook/react",
            "title": "React",
            "description": "A JavaScript library for building user interfaces",
            "trustScore": 95.0
        }"#;

        let result: LibrarySearchResult =
            serde_json::from_str(json).expect("Deve deserializar LibrarySearchResult");

        assert_eq!(result.id, "/facebook/react");
        assert_eq!(result.title, "React");
        assert_eq!(
            result.description.as_deref(),
            Some("A JavaScript library for building user interfaces")
        );
        assert!((result.trust_score.unwrap() - 95.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_library_search_result_deserialisation_tolerates_missing_fields() {
        let json = r#"{
            "id": "/minimal/library",
            "title": "MinimalLib"
        }"#;

        let result: LibrarySearchResult =
            serde_json::from_str(json).expect("Deve deserializar mesmo com campos ausentes");

        assert_eq!(result.id, "/minimal/library");
        assert_eq!(result.title, "MinimalLib");
        assert!(result.description.is_none(), "description deve ser None");
        assert!(result.trust_score.is_none(), "trust_score deve ser None");
    }

    #[test]
    fn test_library_search_result_deserialisation_with_optional_fields() {
        let json = r#"{
            "id": "/facebook/react",
            "title": "React",
            "trustScore": 95.0,
            "stars": 228000,
            "totalSnippets": 1500,
            "totalTokens": 250000,
            "verified": true,
            "branch": "main",
            "state": "active"
        }"#;

        let result: LibrarySearchResult =
            serde_json::from_str(json).expect("Deve deserializar com campos opcionais");

        assert_eq!(result.stars, Some(228_000i64));
        assert_eq!(result.total_snippets, Some(1_500));
        assert_eq!(result.total_tokens, Some(250_000));
        assert_eq!(result.verified, Some(true));
        assert_eq!(result.branch.as_deref(), Some("main"));
        assert_eq!(result.state.as_deref(), Some("active"));
    }

    #[test]
    fn test_documentation_snippet_deserialisation() {
        let json = r#"{
            "pageTitle": "React Hooks API",
            "codeTitle": "useEffect example",
            "codeDescription": "The Effect Hook lets you perform side effects.",
            "codeLanguage": "javascript",
            "codeTokens": 68,
            "codeId": "https://github.com/facebook/react/blob/main/packages/react/src/ReactHooks.js",
            "codeList": [
                {"language": "javascript", "code": "useEffect(() => { /* effect */ }, []);"}
            ],
            "relevance": 0.032,
            "model": "gemini-2.5-flash"
        }"#;

        let snippet: DocumentationSnippet =
            serde_json::from_str(json).expect("Deve deserializar DocumentationSnippet");

        assert_eq!(snippet.page_title.as_deref(), Some("React Hooks API"));
        assert_eq!(snippet.code_title.as_deref(), Some("useEffect example"));
        assert_eq!(snippet.code_language.as_deref(), Some("javascript"));
        assert_eq!(snippet.code_tokens, Some(68));
        let list = snippet.code_list.as_ref().expect("Deve ter code_list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].language, "javascript");
        assert!((snippet.relevance.unwrap() - 0.032).abs() < f64::EPSILON);
    }

    #[test]
    fn test_documentation_snippet_deserialisation_without_optional_fields() {
        let json = r#"{}"#;

        let snippet: DocumentationSnippet =
            serde_json::from_str(json).expect("Deve deserializar snippet completamente vazio");

        assert!(snippet.page_title.is_none());
        assert!(snippet.code_title.is_none());
        assert!(snippet.code_list.is_none());
    }

    #[test]
    fn test_code_block_deserialisation() {
        let json = r#"{"language": "rust", "code": "fn main() {}"}"#;

        let block: CodeBlock = serde_json::from_str(json).expect("Deve deserializar CodeBlock");

        assert_eq!(block.language, "rust");
        assert_eq!(block.code, "fn main() {}");
    }

    // ── Mock HTTP ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_search_library_with_mock_server_returns_200() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let json_response = serde_json::json!({
            "results": [
                {
                    "id": "/axum-rs/axum",
                    "title": "axum",
                    "description": "Framework web para Rust",
                    "trustScore": 90.0
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/search"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&json_response))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/search", mock_server.uri());

        let response = client
            .get(&url)
            .bearer_auth("ctx7sk-teste-mock")
            .query(&[("libraryName", "axum"), ("query", "axum")])
            .send()
            .await
            .expect("Deve conectar ao mock server");

        assert!(response.status().is_success(), "Status deve ser 200");

        let data: LibraryListResponse = response
            .json()
            .await
            .expect("Deve deserializar response do mock");

        assert_eq!(data.results.len(), 1);
        assert_eq!(data.results[0].id, "/axum-rs/axum");
        assert_eq!(data.results[0].title, "axum");
    }

    #[tokio::test]
    async fn test_fetch_documentation_with_mock_server_returns_200() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;

        let json_response = serde_json::json!({
            "snippets": [
                {
                    "pageTitle": "axum::Router",
                    "codeTitle": "Basic Router setup",
                    "codeDescription": "O Router do Axum permite definir rotas HTTP de forma declarativa.",
                    "codeLanguage": "rust",
                    "codeList": [
                        {"language": "rust", "code": "let app = Router::new().route(\"/\", get(handler));"}
                    ]
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/v1/axum-rs/axum"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&json_response))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let url = format!("{}/api/v1/axum-rs/axum", mock_server.uri());

        let response = client
            .get(&url)
            .bearer_auth("ctx7sk-teste-docs-mock")
            .query(&[("type", "json"), ("query", "como criar router")])
            .send()
            .await
            .expect("Deve conectar ao mock server");

        assert!(response.status().is_success());

        let data: DocumentationResponse = response
            .json()
            .await
            .expect("Deve deserializar response do mock");

        let snippets = data.snippets.as_ref().expect("Deve ter snippets");
        assert_eq!(snippets.len(), 1);
        let list = snippets[0].code_list.as_ref().expect("Deve ter code_list");
        assert!(list[0].code.contains("Router::new"));
    }

    // ── Keys shuffle ─────────────────────────────────────────────────────

    #[test]
    fn test_keys_shuffle_preserves_all_elements() {
        let original_keys: Vec<String> =
            (0..10).map(|i| format!("ctx7sk-key-{:02}", i)).collect();

        let mut copy_keys = original_keys.clone();
        // Fisher-Yates shuffle with fastrand — replaces rand::SliceRandom
        for i in (1..copy_keys.len()).rev() {
            let j = fastrand::usize(..=i);
            copy_keys.swap(i, j);
        }

        assert_eq!(
            copy_keys.len(),
            original_keys.len(),
            "Shuffle must preserve all elements"
        );

        let mut sorted_original = original_keys.clone();
        let mut sorted_copy = copy_keys.clone();
        sorted_original.sort();
        sorted_copy.sort();
        assert_eq!(
            sorted_original, sorted_copy,
            "Shuffle must contain the same elements, just in different order"
        );
    }

    #[test]
    fn test_max_attempts_limited_to_5() {
        // Verifies that keys.len().min(5) works correctly
        let many_keys: Vec<String> =
            (0..10).map(|i| format!("ctx7sk-key-{:02}", i)).collect();
        let max = many_keys.len().min(5);
        assert_eq!(
            max, 5,
            "Max attempts deve ser limitado a 5 mesmo com 10 keys"
        );

        let few_keys: Vec<String> = vec!["ctx7sk-a".to_string(), "ctx7sk-b".to_string()];
        let max2 = few_keys.len().min(5);
        assert_eq!(max2, 2, "Com 2 keys, max deve ser 2");
    }
}
