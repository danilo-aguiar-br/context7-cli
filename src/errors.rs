// SPDX-License-Identifier: MIT OR Apache-2.0
//! Error types for the context7-cli library.
//!
//! [`Context7Error`] is used for all structured errors within the library.
//! Binary code uses [`anyhow::Result`] for flexible propagation.
use thiserror::Error;

/// Structured errors for the Context7 API client.
#[derive(Debug, Error)]
pub enum Context7Error {
    /// All available API keys have been exhausted after the given number of attempts.
    #[error("No valid API key available after {attempts} attempts")]
    RetriesExhausted {
        /// Number of attempts made before exhaustion.
        attempts: u32,
    },

    /// All keys failed due to authentication errors (401/403).
    #[error("All API keys failed due to authentication errors")]
    NoValidApiKey,

    /// The API returned an unexpected HTTP status code.
    #[error("Invalid API response: status {status}")]
    InvalidResponse {
        /// HTTP status code returned by the API.
        status: u16,
    },

    /// The API returned HTTP 400 with an error message.
    #[error("API returned error 400: {message}")]
    ApiReturned400 {
        /// Error message returned by the API.
        message: String,
    },

    /// The requested library was not found (HTTP 404).
    #[error("Library not found: {library_id}")]
    LibraryNotFound {
        /// Library identifier that was not found.
        library_id: String,
    },

    /// A keys operation failed (e.g., invalid index, no keys stored).
    /// The caller already printed a user-friendly message; this signals exit code 1.
    #[error("")]
    KeysOperationFailed,
}

impl Context7Error {
    /// Maps each error variant to a BSD-style exit code (sysexits.h).
    ///
    /// | Code | Constant       | Meaning                          |
    /// |------|----------------|----------------------------------|
    /// |   1  | generic        | Unspecified runtime error         |
    /// |  65  | EX_DATAERR     | Invalid input data               |
    /// |  66  | EX_NOINPUT     | Requested resource not found     |
    /// |  69  | EX_UNAVAILABLE | Service unavailable after retry  |
    /// |  74  | EX_IOERR       | I/O or network error             |
    /// |  77  | EX_NOPERM      | Permission / authentication denied|
    #[must_use]
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::RetriesExhausted { .. } => 69,
            Self::NoValidApiKey => 77,
            Self::InvalidResponse { .. } => 74,
            Self::ApiReturned400 { .. } => 65,
            Self::LibraryNotFound { .. } => 66,
            Self::KeysOperationFailed => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_no_api_keys_display() {
        let err = Context7Error::NoValidApiKey;
        let message = err.to_string();
        assert!(
            !message.is_empty(),
            "NoValidApiKey must have non-empty message"
        );
        assert!(
            message.to_lowercase().contains("key")
                || message.to_lowercase().contains("api")
                || message.to_lowercase().contains("auth"),
            "Message must mention key/api/auth, got: {message}"
        );
    }

    #[test]
    fn test_error_retries_exhausted_contains_attempts_count() {
        let err = Context7Error::RetriesExhausted { attempts: 3 };
        let message = err.to_string();
        assert!(
            message.contains('3'),
            "Message must contain number of attempts (3), got: {message}"
        );
    }

    #[test]
    fn test_error_invalid_response_contains_status() {
        let err = Context7Error::InvalidResponse { status: 500 };
        let message = err.to_string();
        assert!(
            message.contains("500"),
            "Message must contain status code, got: {message}"
        );
    }

    #[test]
    fn test_error_api_400_contains_error_text() {
        let err = Context7Error::ApiReturned400 {
            message: "Invalid parameter".to_string(),
        };
        let message = err.to_string();
        assert!(
            message.contains("Invalid parameter"),
            "Message must contain error text, got: {message}"
        );
    }

    #[test]
    fn test_result_alias_propagates_context7_error() {
        fn fail() -> Result<(), Context7Error> {
            Err(Context7Error::NoValidApiKey)
        }
        let result: Result<(), Context7Error> = fail();
        assert!(result.is_err(), "Result must be Err");
        let err = result.unwrap_err();
        assert!(matches!(err, Context7Error::NoValidApiKey));
    }
}
