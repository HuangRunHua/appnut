//! Authentication trait for the DSL framework.
//!
//! The DSL framework does NOT depend on any specific auth module.
//! It only knows this trait. The concrete implementation is injected
//! at startup time.

use axum::http::HeaderMap;

use crate::ServiceError;

/// Pluggable authenticator. The DSL framework calls this for every
/// endpoint that has a `#[permission("...")]` annotation.
///
/// The check receives the request headers (for extracting tokens)
/// and the permission string from the DSL annotation.
pub trait Authenticator: Send + Sync + 'static {
    /// Authenticate a request and check the given permission.
    ///
    /// - `headers`: the HTTP request headers
    /// - `permission`: the string from `#[permission("module:resource:action")]`
    /// - Returns `Ok(())` if allowed, `Err(ServiceError)` if denied.
    fn check(
        &self,
        headers: &HeaderMap,
        permission: &str,
    ) -> Result<(), ServiceError>;
}

/// A no-op authenticator that allows everything. Used for testing
/// and for public-only APIs.
pub struct AllowAll;

impl Authenticator for AllowAll {
    fn check(&self, _headers: &HeaderMap, _permission: &str) -> Result<(), ServiceError> {
        Ok(())
    }
}

/// An authenticator that denies everything. Used for testing.
pub struct DenyAll;

impl Authenticator for DenyAll {
    fn check(&self, _headers: &HeaderMap, _permission: &str) -> Result<(), ServiceError> {
        Err(ServiceError::PermissionDenied("access denied".into()))
    }
}

/// JWT-based authenticator that verifies `Authorization: Bearer <token>` headers
/// using HMAC-SHA256.
///
/// The `permission` parameter is currently ignored — any valid, non-expired
/// token is accepted.
pub struct JwtAuthenticator {
    decoding_key: jsonwebtoken::DecodingKey,
    validation: jsonwebtoken::Validation,
}

impl JwtAuthenticator {
    /// Create a new authenticator with an HMAC-SHA256 secret.
    pub fn new(secret: &str) -> Self {
        Self {
            decoding_key: jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            validation: jsonwebtoken::Validation::default(),
        }
    }
}

impl Authenticator for JwtAuthenticator {
    fn check(&self, headers: &HeaderMap, _permission: &str) -> Result<(), ServiceError> {
        let header_value = headers
            .get(axum::http::header::AUTHORIZATION)
            .ok_or_else(|| ServiceError::Unauthorized("missing Authorization header".into()))?;

        let auth_str = header_value
            .to_str()
            .map_err(|_| ServiceError::Unauthorized("invalid Authorization header".into()))?;

        let token = auth_str
            .strip_prefix("Bearer ")
            .ok_or_else(|| ServiceError::Unauthorized("missing Bearer prefix".into()))?;

        jsonwebtoken::decode::<serde_json::Value>(token, &self.decoding_key, &self.validation)
            .map_err(|e| ServiceError::Unauthorized(format!("invalid token: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    const TEST_SECRET: &str = "test-jwt-secret";

    #[derive(Serialize)]
    struct TestClaims {
        sub: String,
        exp: i64,
    }

    fn make_token(secret: &str, exp: i64) -> String {
        let claims = TestClaims {
            sub: "alice".into(),
            exp,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn headers_with_token(token: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::AUTHORIZATION,
            format!("Bearer {token}").parse().unwrap(),
        );
        headers
    }

    #[test]
    fn valid_token_passes() {
        let auth = JwtAuthenticator::new(TEST_SECRET);
        let exp = chrono::Utc::now().timestamp() + 3600;
        let token = make_token(TEST_SECRET, exp);
        assert!(auth.check(&headers_with_token(&token), "any:perm").is_ok());
    }

    #[test]
    fn missing_authorization_header_fails() {
        let auth = JwtAuthenticator::new(TEST_SECRET);
        let err = auth.check(&HeaderMap::new(), "any:perm").unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized(_)));
    }

    #[test]
    fn invalid_token_fails() {
        let auth = JwtAuthenticator::new(TEST_SECRET);
        let err = auth
            .check(&headers_with_token("not.a.valid.token"), "any:perm")
            .unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized(_)));
    }

    #[test]
    fn expired_token_fails() {
        let auth = JwtAuthenticator::new(TEST_SECRET);
        let exp = chrono::Utc::now().timestamp() - 120;
        let token = make_token(TEST_SECRET, exp);
        let err = auth
            .check(&headers_with_token(&token), "any:perm")
            .unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized(_)));
    }
}
