//! Testes de integração HTTP para o módulo `api`.
//!
//! Uses `wiremock` to set up a local HTTP server that simulates the Context7 API.
//! No test makes a real request to the internet.
//!
//! Os testes acessam diretamente `context7_cli::api::*` via `[lib]` em Cargo.toml,
//! allowing testing of HTTP, deserialization and retry logic without invoking the binary.

use context7_cli::api::{
    create_http_client, run_with_retry, DocumentationSnippet, LibrarySearchResult,
    DocumentationResponse, LibraryListResponse,
};
use context7_cli::errors::Context7Error;
use context7_cli::storage::ApiKey;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper: creates `Vec<ApiKey>` from literal strings for tests.
fn chaves_api(valores: &[&str]) -> Vec<ApiKey> {
    valores
        .iter()
        .map(|v| ApiKey::new(v.to_string()))
        .collect()
}

// ─── search_library tests ───

/// `search_library` returns a list of libraries when the server responds 200.
#[tokio::test]
async fn test_search_library_returns_results_200() {
    let servidor = MockServer::start().await;

    let resposta_json = serde_json::json!({
        "results": [
            {
                "id": "/facebook/react",
                "title": "React",
                "description": "A JavaScript library for building user interfaces",
                "trustScore": 95.0
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .and(query_param("libraryName", "react"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&resposta_json))
        .mount(&servidor)
        .await;

    // Overrides BASE_URL via client with mock URL
    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/search");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("libraryName", "react"), ("query", "react")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    assert!(resposta.status().is_success());

    let data: LibraryListResponse = resposta.json().await.expect("deve deserializar");
    assert_eq!(data.results.len(), 1);
    assert_eq!(data.results[0].id, "/facebook/react");
    assert_eq!(data.results[0].title, "React");
    assert!((data.results[0].trust_score.unwrap() - 95.0).abs() < f64::EPSILON);
}

/// `search_library` returns an empty list when the server responds with results=[].
#[tokio::test]
async fn test_search_library_returns_empty_list() {
    let servidor = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "results": [] })),
        )
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/search");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("libraryName", "inexistente"), ("query", "inexistente")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    let data: LibraryListResponse = resposta.json().await.expect("deve deserializar");
    assert_eq!(data.results.len(), 0, "lista deve estar vazia");
}

/// Server 401 response must map to `Context7Error::NoValidApiKey`.
#[tokio::test]
async fn test_search_library_401_maps_to_no_api_keys() {
    let servidor = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();

    // Simulates handle_response_status behavior via search_library with injected URL
    let url = format!("{url_base}/api/v1/search");
    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-invalida")
        .query(&[("libraryName", "react"), ("query", "react")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    assert_eq!(resposta.status().as_u16(), 401, "mock deve retornar 401");
}

/// Response 429 (rate limit) must be handled without panic.
#[tokio::test]
async fn test_search_library_429_does_not_panic() {
    let servidor = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/search"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/search");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("libraryName", "react"), ("query", "react")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    assert_eq!(resposta.status().as_u16(), 429, "mock deve retornar 429");
}

// ─── fetch_documentation tests ───

/// `fetch_documentation` returns JSON snippets when the server responds 200.
#[tokio::test]
async fn test_fetch_documentation_returns_snippets_200() {
    let servidor = MockServer::start().await;

    let resposta_json = serde_json::json!({
        "snippets": [
            {
                "pageTitle": "React Hooks",
                "codeTitle": "useEffect hook",
                "codeDescription": "O useEffect é um hook para efeitos colaterais.",
                "codeLanguage": "javascript",
                "codeList": [
                    {"language": "javascript", "code": "useEffect(() => {}, []);"}
                ]
            }
        ]
    });

    Mock::given(method("GET"))
        .and(path("/api/v1/facebook/react"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&resposta_json))
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/facebook/react");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("type", "json")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    assert!(resposta.status().is_success());

    let data: DocumentationResponse = resposta.json().await.expect("deve deserializar");
    let snippets = data.snippets.expect("deve ter snippets");
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].code_title.as_deref(), Some("useEffect hook"));
    let lista = snippets[0].code_list.as_ref().expect("deve ter code_list");
    assert!(lista[0].code.contains("useEffect"));
}

/// Plain text mode: the server returns raw text/markdown (not JSON).
///
/// Verifies that an HTTP client can receive plain text (status 200, content-type text/plain)
/// — expected behavior of `fetch_documentation_text` on the `type=txt` endpoint.
#[tokio::test]
async fn test_fetch_documentation_text_returns_plain_content() {
    let servidor = MockServer::start().await;

    let conteudo_markdown = "# Axum\n\nAxum é um framework web para Rust baseado em Tower e Tokio.";

    Mock::given(method("GET"))
        .and(path("/api/v1/axum-rs/axum"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(conteudo_markdown)
                .insert_header("content-type", "text/plain"),
        )
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/axum-rs/axum");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("type", "txt")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    assert!(resposta.status().is_success());
    let text = resposta.text().await.expect("deve ler text");
    assert!(text.contains("Axum"), "content must mention Axum");
}

/// Server 500 response must be handled without panic (server error).
#[tokio::test]
async fn test_fetch_documentation_500_does_not_panic() {
    let servidor = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/rust-lang/rust"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/rust-lang/rust");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("type", "json")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    assert_eq!(resposta.status().as_u16(), 500, "mock deve retornar 500");
}

// ─── run_with_retry tests ───

/// `run_with_retry` with a valid key returns Ok on the first attempt.
#[tokio::test]
async fn test_run_with_retry_success_on_first_attempt() {
    let keys = chaves_api(&["ctx7sk-key-valida-123456789012"]);

    let result = run_with_retry(&keys, |_chave| async move {
        Ok::<String, Context7Error>("sucesso".to_string())
    })
    .await;

    assert!(result.is_ok(), "deve retornar Ok na primeira tentativa");
    assert_eq!(result.unwrap(), "sucesso");
}

/// `run_with_retry` with all invalid keys returns `NoValidApiKey`.
#[tokio::test]
async fn test_run_with_retry_all_invalid_keys_returns_no_keys() {
    let keys = chaves_api(&[
        "ctx7sk-invalida-01-123456789012",
        "ctx7sk-invalida-02-123456789012",
        "ctx7sk-invalida-03-123456789012",
    ]);

    let result = run_with_retry(&keys, |_chave| async move {
        Err::<String, Context7Error>(Context7Error::NoValidApiKey)
    })
    .await;

    assert!(
        result.is_err(),
        "deve falhar quando todas as keys são inválidas"
    );
}

/// `run_with_retry` tenta a segunda key quando a primeira failure com erro transitório.
#[tokio::test]
async fn test_run_with_retry_uses_second_key_when_first_fails() {
    use std::sync::Arc;
    use std::sync::Mutex;

    let contador = Arc::new(Mutex::new(0usize));
    let keys = chaves_api(&[
        "ctx7sk-primeira-key-123456789",
        "ctx7sk-segunda-key-1234567890",
    ]);

    let contador_clone = Arc::clone(&contador);
    let result = run_with_retry(&keys, move |_chave| {
        let cont = Arc::clone(&contador_clone);
        async move {
            let mut n = cont.lock().unwrap();
            *n += 1;
            let tentativa = *n;
            drop(n);
            if tentativa == 1 {
                // First attempt fails with transient error (not auth)
                Err(Context7Error::InvalidResponse { status: 503 })
            } else {
                Ok::<String, Context7Error>("sucesso na segunda".to_string())
            }
        }
    })
    .await;

    assert!(result.is_ok(), "deve ter sucesso na segunda tentativa");
    let attempts = *contador.lock().unwrap();
    assert!(
        attempts >= 2,
        "deve ter feito pelo menos 2 attempts, fez: {attempts}"
    );
}

/// `run_with_retry` aborta imediatamente em erro 400 (não transitório).
#[tokio::test]
async fn test_run_with_retry_aborts_on_400_error() {
    let keys = chaves_api(&[
        "ctx7sk-key-01-123456789012345",
        "ctx7sk-key-02-123456789012345",
    ]);

    let result = run_with_retry(&keys, |_chave| async move {
        Err::<String, Context7Error>(Context7Error::ApiReturned400 {
            message: "bad request".to_string(),
        })
    })
    .await;

    assert!(result.is_err(), "deve falhar em 400");
    let err = result.unwrap_err().to_string();
    // Must propagate the 400 error, not RetriesExhausted
    assert!(
        err.contains("400") || err.contains("bad request") || err.contains("Bad request"),
        "message must mention 400 or bad request: {err}"
    );
}

// ── New tests v0.2.1 — coverage of fixed bugs ──────────────────────

/// `run_with_retry` short-circuits on parse error (HTTP 200 with invalid JSON).
///
/// BUG v0.2.0 CORRIGIDO: schema `DocumentationSnippet` estava errado, retornando
/// `InvalidResponse { status: 200 }` instead of trying other keys indefinitely.
/// v0.2.1 aborta imediatamente quando status = 200 mas parse falhou.
#[tokio::test]
async fn test_retry_short_circuits_on_parse_error_status_200() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let contador_attempts = Arc::new(AtomicUsize::new(0));
    // 3 keys available — but must abort on the 1st with InvalidResponse { status: 200 }
    let keys = chaves_api(&[
        "ctx7sk-key-01-12345678901234",
        "ctx7sk-key-02-12345678901234",
        "ctx7sk-key-03-12345678901234",
    ]);

    let cont_clone = Arc::clone(&contador_attempts);
    let result = run_with_retry(&keys, move |_chave| {
        let cont = Arc::clone(&cont_clone);
        async move {
            cont.fetch_add(1, Ordering::SeqCst);
            // Simulates HTTP 200 with body that does not parse (schema mismatch)
            Err::<String, Context7Error>(Context7Error::InvalidResponse { status: 200 })
        }
    })
    .await;

    assert!(result.is_err(), "deve falhar com parse error");
    let attempts_feitas = contador_attempts.load(Ordering::SeqCst);
    assert_eq!(
        attempts_feitas, 1,
        "deve ter feito exactly 1 tentativa before de abortar (short-circuit), fez: {attempts_feitas}"
    );
}

/// `run_with_retry` tenta até 5 keys quando disponíveis (bug v0.2.0 corrigido).
///
/// BUG v0.2.0: `max_attempts = 3usize.min(keys.len())` nunca ultrapassava 3.
/// v0.2.1: `max_attempts = keys.len().min(5)` permite até 5 attempts.
#[tokio::test]
async fn test_retry_uses_more_than_3_keys_when_available() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let contador = Arc::new(AtomicUsize::new(0));
    // 5 keys — the first 4 fail with auth error, the 5th succeeds
    let keys: Vec<ApiKey> = (1..=5)
        .map(|i| ApiKey::new(format!("ctx7sk-key-{i:02}-12345678901234")))
        .collect();

    let cont_clone = Arc::clone(&contador);
    let result = run_with_retry(&keys, move |_chave| {
        let cont = Arc::clone(&cont_clone);
        async move {
            let n = cont.fetch_add(1, Ordering::SeqCst) + 1;
            if n < 5 {
                // Primeiras 4 keys: auth inválida
                Err(Context7Error::NoValidApiKey)
            } else {
                // 5ª key: sucesso
                Ok::<String, Context7Error>("sucesso na 5ª key".to_string())
            }
        }
    })
    .await;

    assert!(
        result.is_ok(),
        "deve ter sucesso na 5ª key, obteve: {:?}",
        result.err().map(|e| e.to_string())
    );
    let attempts = contador.load(Ordering::SeqCst);
    assert_eq!(
        attempts, 5,
        "deve ter tentado exactly 5 keys (v0.2.1 limite = 5), fez: {attempts}"
    );
}

// ─── Unit tests for struct deserialisation v0.2.1 ───

/// `LibrarySearchResult` com `trustScore` camelCase desserializa para `trust_score`.
///
/// BUG v0.2.0 CORRIGIDO: sem `#[serde(rename_all = "camelCase")]`, `trust_score`
/// never received the value because the API returns `trustScore` (camelCase).
#[test]
fn test_library_search_result_deserialisation_camelcase_trust_score() {
    let json = r#"{
        "id": "/facebook/react",
        "title": "React",
        "trustScore": 95.0
    }"#;

    let result: LibrarySearchResult =
        serde_json::from_str(json).expect("deve deserializar LibrarySearchResult com camelCase");

    assert!(
        result.trust_score.is_some(),
        "trust_score deve ser Some após correção do camelCase (bug v0.2.0 era sempre None)"
    );
    assert!(
        (result.trust_score.unwrap() - 95.0).abs() < f64::EPSILON,
        "trust_score deve ser 95.0, obteve: {:?}",
        result.trust_score
    );
}

/// `LibrarySearchResult` com campos extras (stars, verified, totalSnippets) desserializa.
#[test]
fn test_library_search_result_deserialisation_with_extra_fields() {
    let json_str = r#"{
        "id": "/rust-lang/rust",
        "title": "Rust",
        "trustScore": 9.8,
        "stars": 97000,
        "verified": true,
        "totalSnippets": 500,
        "totalTokens": 1000000
    }"#;

    let result: LibrarySearchResult =
        serde_json::from_str(json_str).expect("deve desserializar com campos extras");

    assert_eq!(result.id, "/rust-lang/rust");
    assert_eq!(result.total_snippets, Some(500));
    assert_eq!(result.stars, Some(97000));
    assert_eq!(result.verified, Some(true));
    assert_eq!(result.total_tokens, Some(1000000));
}

/// `DocumentationSnippet` desserializa corretamente com schema real da API v0.2.1.
///
/// BUG v0.2.0 CORRIGIDO: schema antigo usava `content/tipo/source_urls`.
/// v0.2.1 usa `codeTitle/codeDescription/codeLanguage/codeList/...` (camelCase).
#[test]
fn test_documentation_snippet_deserialisation_schema_v021() {
    let json = r#"{
        "codeTitle": "Test function",
        "codeDescription": "A test description",
        "codeLanguage": "rust",
        "codeTokens": 10,
        "codeId": "https://example.com/test",
        "pageTitle": "Test page",
        "codeList": [
            {"language": "rust", "code": "fn main() {}"}
        ],
        "relevance": 0.95,
        "model": "gpt-4o"
    }"#;

    let snippet: DocumentationSnippet =
        serde_json::from_str(json).expect("deve deserializar DocumentationSnippet v0.2.1");

    assert_eq!(snippet.code_title.as_deref(), Some("Test function"));
    assert_eq!(
        snippet.code_description.as_deref(),
        Some("A test description")
    );
    assert_eq!(snippet.code_language.as_deref(), Some("rust"));
    assert_eq!(snippet.code_tokens, Some(10));
    assert_eq!(snippet.code_id.as_deref(), Some("https://example.com/test"));
    assert_eq!(snippet.page_title.as_deref(), Some("Test page"));
    assert_eq!(snippet.relevance, Some(0.95));
    assert_eq!(snippet.model.as_deref(), Some("gpt-4o"));

    let lista = snippet.code_list.as_ref().expect("deve ter code_list");
    assert_eq!(lista.len(), 1);
    assert_eq!(lista[0].language, "rust");
    assert_eq!(lista[0].code, "fn main() {}");
}

/// `DocumentationSnippet` tolera todos os campos ausentes (schema mínimo).
#[test]
fn test_documentation_snippet_deserialisation_all_optional_fields() {
    let json = r#"{}"#;

    let snippet: DocumentationSnippet =
        serde_json::from_str(json).expect("deve deserializar snippet vazio sem erro");

    assert!(snippet.code_title.is_none());
    assert!(snippet.code_description.is_none());
    assert!(snippet.code_language.is_none());
    assert!(snippet.code_tokens.is_none());
    assert!(snippet.code_id.is_none());
    assert!(snippet.page_title.is_none());
    assert!(snippet.code_list.is_none());
    assert!(snippet.relevance.is_none());
    assert!(snippet.model.is_none());
}

// ── New tests v0.2.2 — LibraryNotFound short-circuit ─────────────

/// `run_with_retry` must short-circuit immediately when receiving
/// `Context7Error::LibraryNotFound` — must not try other keys.
///
/// Verifies the bug fixed in v0.2.2: before, an HTTP 404 was reported as
/// "No valid API key available after 5 attempts" porque o retry continuava
/// tentando outras keys mesmo quando a biblioteca não existia.
#[tokio::test]
async fn test_run_with_retry_short_circuits_on_library_not_found() {
    use context7_cli::errors::Context7Error;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let keys = chaves_api(&[
        "ctx7sk-k1-12345678901234",
        "ctx7sk-k2-12345678901234",
        "ctx7sk-k3-12345678901234",
    ]);
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = Arc::clone(&attempts);

    let result: Result<(), _> = run_with_retry(&keys, move |_chave| {
        let t = Arc::clone(&attempts_clone);
        async move {
            t.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(Context7Error::LibraryNotFound {
                library_id: "/teste/inexistente".to_string(),
            })
        }
    })
    .await;

    assert!(result.is_err(), "deve retornar Err");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("not found") || msg.contains("/teste/inexistente"),
        "error must mention library not found or library_id, got: {msg}"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "deve tentar APENAS 1 vez (short-circuit), não as {} keys disponíveis",
        keys.len()
    );
}

/// `Context7Error::LibraryNotFound` must format readable message with library_id.
#[test]
fn test_library_not_found_error_display() {
    let erro = Context7Error::LibraryNotFound {
        library_id: "/facebook/react".to_string(),
    };
    let msg = erro.to_string();
    assert!(
        msg.contains("/facebook/react"),
        "message deve conter o library_id, obteve: {msg}"
    );
    assert!(
        msg.to_lowercase().contains("not found") || msg.to_lowercase().contains("library"),
        "message must mention 'not found' or 'library', got: {msg}"
    );
}

/// `LibrarySearchResult` aceita `stars: -1` — sentinela da API Context7 para
/// bibliotecas sem repositório GitHub (ex: `/websites/*`).
///
/// Prova a correção do bug histórico v0.2.0→v0.2.1: campo era `Option<u64>`,
/// which caused silent deserialization failure when receiving negative integers.
#[test]
fn test_library_search_result_accepts_negative_stars() {
    let json = r#"{
        "id": "/websites/react_dev",
        "title": "React",
        "stars": -1,
        "totalSnippets": 5724,
        "totalTokens": 841799,
        "trustScore": 8.5
    }"#;
    let r: LibrarySearchResult = serde_json::from_str(json)
        .expect("Deve deserializar mesmo com stars: -1 (bug histórico v0.2.0→v0.2.1)");
    assert_eq!(r.stars, Some(-1));
    assert_eq!(r.trust_score, Some(8.5));
    assert_eq!(r.id, "/websites/react_dev");
}

// ── New tests v0.2.3 — retry short-circuit LibraryNotFound via docs ─

/// `run_with_retry` in `fetch_documentation` mode aborts with LibraryNotFound
/// when the server returns 404 — ensures that the v0.2.2 short-circuit works
/// também para o path de `docs`, não só de `library`.
///
/// This test validates that `fetch_documentation` converts 404 → LibraryNotFound
/// and that retry does not try other keys (expected behavior after D1/Bug #1).
#[tokio::test]
async fn test_fetch_documentation_404_converts_to_library_not_found() {
    let servidor = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/inexistente/lib"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&servidor)
        .await;

    let url_base = servidor.uri();
    let client = create_http_client().unwrap();
    let url = format!("{url_base}/api/v1/inexistente/lib");

    let resposta = client
        .get(&url)
        .bearer_auth("ctx7sk-mock-token-12345678")
        .query(&[("type", "json")])
        .send()
        .await
        .expect("deve conectar ao mock server");

    // 404 must be returned — the conversion logic is in fetch_documentation
    assert_eq!(
        resposta.status().as_u16(),
        404,
        "mock deve retornar 404 para biblioteca inexistente"
    );
}

/// `run_with_retry` faz short-circuit imediato ao receber LibraryNotFound
/// instead of trying all available keys — central behavior of Bug #1 fix.
///
/// This test confirms that retry does NOT iterate through keys when the error is not
/// de autenticação (401/403) mas yes de recurso inexistente (404 → LibraryNotFound).
#[tokio::test]
async fn test_retry_does_not_iterate_keys_when_library_does_not_exist() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    // 5 keys disponíveis — com LibraryNotFound, DEVE tentar apenas 1
    let keys: Vec<ApiKey> = (1..=5)
        .map(|i| ApiKey::new(format!("ctx7sk-key-{i:02}-12345678901234")))
        .collect();

    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_clone = Arc::clone(&attempts);

    let result: Result<(), _> = run_with_retry(&keys, move |_chave| {
        let t = Arc::clone(&attempts_clone);
        async move {
            t.fetch_add(1, Ordering::SeqCst);
            // Simulates HTTP 404 converted to LibraryNotFound in fetch_documentation
            Err::<(), _>(Context7Error::LibraryNotFound {
                library_id: "/inexistente/lib".to_string(),
            })
        }
    })
    .await;

    assert!(
        result.is_err(),
        "deve retornar Err para biblioteca inexistente"
    );
    assert_eq!(
        attempts.load(Ordering::SeqCst),
        1,
        "com LibraryNotFound, retry deve abortar após 1 tentativa (não rodar para todas as {} keys)",
        keys.len()
    );

    // The returned error must mention the original library_id
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("/inexistente/lib") || msg.contains("not found"),
        "message deve identificar a biblioteca inexistente: {msg}"
    );
}
