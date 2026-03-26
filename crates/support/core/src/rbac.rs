//! RBAC (Role-Based Access Control) — role definitions, permission matching,
//! and an `Authenticator` implementation that verifies JWT + checks permissions.

use std::collections::HashMap;
use std::sync::Arc;

use axum::http::HeaderMap;
use serde::Deserialize;

use crate::ServiceError;
use crate::auth::{AllowAll, Authenticator};

// ── Role trait ──────────────────────────────────────────────────────

/// Trait that role enums implement to integrate with RBAC.
///
/// ```ignore
/// enum MyRole { Admin, User }
///
/// impl RolePermissions for MyRole {
///     fn role_name(&self) -> &str {
///         match self {
///             Self::Admin => "admin",
///             Self::User  => "user",
///         }
///     }
///     fn granted_permissions(&self) -> &[&str] {
///         match self {
///             Self::Admin => &["*:*:*"],
///             Self::User  => &["twitter:tweet:create", "twitter:tweet:read"],
///         }
///     }
/// }
/// ```
pub trait RolePermissions {
    fn role_name(&self) -> &str;
    fn granted_permissions(&self) -> &[&str];
}

// ── PermissionMap ───────────────────────────────────────────────────

/// Stores role → granted permission patterns.
///
/// Build from an array of [`RolePermissions`] variants, then pass to
/// [`RbacAuthenticator`].
#[derive(Debug, Clone)]
pub struct PermissionMap {
    roles: HashMap<String, Vec<String>>,
}

impl PermissionMap {
    pub fn new() -> Self {
        Self {
            roles: HashMap::new(),
        }
    }

    /// Register a role with its granted permission patterns.
    pub fn add_role(&mut self, role_name: &str, permissions: &[&str]) {
        self.roles.insert(
            role_name.to_string(),
            permissions.iter().map(|s| s.to_string()).collect(),
        );
    }

    /// Build from an iterator of [`RolePermissions`] values.
    pub fn from_roles(roles: &[&dyn RolePermissions]) -> Self {
        let mut map = Self::new();
        for role in roles {
            map.add_role(role.role_name(), role.granted_permissions());
        }
        map
    }

    /// Check whether `role_name` is granted `required_permission`.
    pub fn is_allowed(&self, role_name: &str, required_permission: &str) -> bool {
        let Some(granted) = self.roles.get(role_name) else {
            return false;
        };
        granted
            .iter()
            .any(|pattern| permission_matches(required_permission, pattern))
    }

    /// Return all permission patterns granted to `role_name`.
    pub fn permissions_for(&self, role_name: &str) -> Vec<String> {
        self.roles.get(role_name).cloned().unwrap_or_default()
    }
}

impl Default for PermissionMap {
    fn default() -> Self {
        Self::new()
    }
}

// ── Permission matching ─────────────────────────────────────────────

/// Check if a granted permission pattern matches a required permission.
///
/// Both strings use `module:resource:action` format.
/// `*` in the granted pattern matches any single segment.
///
/// Examples:
///   `permission_matches("twitter:tweet:create", "twitter:tweet:create")` → true
///   `permission_matches("twitter:tweet:create", "twitter:*:*")` → true
///   `permission_matches("twitter:tweet:create", "*:*:*")` → true
///   `permission_matches("twitter:tweet:create", "shop:*:*")` → false
pub fn permission_matches(required: &str, granted: &str) -> bool {
    let req_parts: Vec<&str> = required.split(':').collect();
    let grant_parts: Vec<&str> = granted.split(':').collect();

    if req_parts.len() != grant_parts.len() {
        return false;
    }

    req_parts
        .iter()
        .zip(grant_parts.iter())
        .all(|(req, grant)| *grant == "*" || req == grant)
}

// ── RbacAuthenticator ───────────────────────────────────────────────

/// Minimal JWT claims — only the fields RBAC needs.
/// Application-specific claims (name, etc.) are ignored here.
#[derive(Debug, Deserialize)]
struct RbacClaims {
    #[serde(default)]
    role: Option<String>,
}

/// RBAC authenticator — verifies JWT signature + checks role permissions.
///
/// Workflow per request:
/// 1. Extract `Bearer` token from `Authorization` header
/// 2. Decode JWT using the provided HMAC key
/// 3. Read the `role` claim
/// 4. Look up role in [`PermissionMap`]
/// 5. Check if any granted pattern matches the required permission
pub struct RbacAuthenticator {
    decoding_key: jsonwebtoken::DecodingKey,
    validation: jsonwebtoken::Validation,
    permissions: PermissionMap,
}

impl RbacAuthenticator {
    pub fn new(secret: &str, permissions: PermissionMap) -> Self {
        Self {
            decoding_key: jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            validation: jsonwebtoken::Validation::default(),
            permissions,
        }
    }

    fn extract_token<'a>(&self, headers: &'a HeaderMap) -> Result<&'a str, ServiceError> {
        headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or_else(|| {
                ServiceError::Unauthorized("missing or invalid authorization header".into())
            })
    }
}

impl Authenticator for RbacAuthenticator {
    fn check(&self, headers: &HeaderMap, permission: &str) -> Result<(), ServiceError> {
        let token = self.extract_token(headers)?;

        let token_data =
            jsonwebtoken::decode::<RbacClaims>(token, &self.decoding_key, &self.validation)
                .map_err(|e| ServiceError::Unauthorized(format!("invalid token: {e}")))?;

        let role = token_data.claims.role.as_deref().unwrap_or("user");

        if !self.permissions.is_allowed(role, permission) {
            return Err(ServiceError::PermissionDenied(format!(
                "role '{}' lacks permission '{}'",
                role, permission
            )));
        }

        Ok(())
    }
}

// ── Auth mode resolution ────────────────────────────────────────────

/// Environment variable that controls auth mode.
pub const AUTH_MODE_ENV: &str = "APPNUT_AUTH_MODE";

/// Resolve the authenticator based on the `APPNUT_AUTH_MODE` env var.
///
/// - `allow_all` → returns [`AllowAll`] (for development)
/// - anything else / unset → returns the provided RBAC authenticator
///
/// Logs the chosen mode via `tracing`.
pub fn resolve_auth_mode(rbac: RbacAuthenticator) -> Arc<dyn Authenticator> {
    if std::env::var(AUTH_MODE_ENV).ok().as_deref() == Some("allow_all") {
        tracing::warn!("[AUTH] mode=allow_all (DEVELOPMENT ONLY)");
        Arc::new(AllowAll)
    } else {
        tracing::info!("[AUTH] mode=rbac");
        Arc::new(rbac)
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(permission_matches(
            "twitter:tweet:create",
            "twitter:tweet:create"
        ));
    }

    #[test]
    fn wildcard_action() {
        assert!(permission_matches(
            "twitter:tweet:create",
            "twitter:tweet:*"
        ));
    }

    #[test]
    fn wildcard_resource_and_action() {
        assert!(permission_matches("twitter:tweet:create", "twitter:*:*"));
    }

    #[test]
    fn super_admin() {
        assert!(permission_matches("twitter:tweet:create", "*:*:*"));
        assert!(permission_matches("shop:order:delete", "*:*:*"));
    }

    #[test]
    fn wrong_module() {
        assert!(!permission_matches("twitter:tweet:create", "shop:*:*"));
    }

    #[test]
    fn wrong_action() {
        assert!(!permission_matches(
            "twitter:tweet:create",
            "twitter:tweet:delete"
        ));
    }

    #[test]
    fn segment_count_mismatch() {
        assert!(!permission_matches("twitter:tweet", "twitter:tweet:create"));
    }

    #[test]
    fn permission_map_basic() {
        let mut map = PermissionMap::new();
        map.add_role("admin", &["*:*:*"]);
        map.add_role("user", &["twitter:tweet:create", "twitter:tweet:read"]);

        assert!(map.is_allowed("admin", "twitter:tweet:delete"));
        assert!(map.is_allowed("user", "twitter:tweet:create"));
        assert!(!map.is_allowed("user", "twitter:tweet:delete"));
        assert!(!map.is_allowed("guest", "twitter:tweet:read"));
    }

    #[test]
    fn permission_map_wildcard_role() {
        let mut map = PermissionMap::new();
        map.add_role("moderator", &["twitter:*:read", "twitter:*:list"]);

        assert!(map.is_allowed("moderator", "twitter:tweet:read"));
        assert!(map.is_allowed("moderator", "twitter:user:list"));
        assert!(!map.is_allowed("moderator", "twitter:tweet:create"));
    }

    #[test]
    fn permissions_for_role() {
        let mut map = PermissionMap::new();
        map.add_role("user", &["a:b:c", "d:e:f"]);

        let perms = map.permissions_for("user");
        assert_eq!(perms.len(), 2);
        assert!(perms.contains(&"a:b:c".to_string()));

        let empty = map.permissions_for("nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn rbac_authenticator_missing_header() {
        let auth = RbacAuthenticator::new("secret", PermissionMap::new());
        let headers = HeaderMap::new();
        let result = auth.check(&headers, "any:perm:here");
        assert!(result.is_err());
    }

    #[test]
    fn rbac_authenticator_invalid_token() {
        let auth = RbacAuthenticator::new("secret", PermissionMap::new());
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer invalid.token".parse().unwrap());
        let result = auth.check(&headers, "any:perm:here");
        assert!(result.is_err());
    }

    #[test]
    fn rbac_authenticator_valid_token_allowed() {
        let secret = "test-secret-key";
        let mut map = PermissionMap::new();
        map.add_role("admin", &["*:*:*"]);

        let auth = RbacAuthenticator::new(secret, map);

        let claims = serde_json::json!({
            "sub": "alice", "role": "admin",
            "iat": chrono::Utc::now().timestamp(),
            "exp": chrono::Utc::now().timestamp() + 3600,
        });
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        assert!(auth.check(&headers, "twitter:user:list").is_ok());
    }

    #[test]
    fn rbac_authenticator_valid_token_denied() {
        let secret = "test-secret-key";
        let mut map = PermissionMap::new();
        map.add_role("user", &["twitter:tweet:read"]);

        let auth = RbacAuthenticator::new(secret, map);

        let claims = serde_json::json!({
            "sub": "alice", "role": "user",
            "iat": chrono::Utc::now().timestamp(),
            "exp": chrono::Utc::now().timestamp() + 3600,
        });
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        assert!(auth.check(&headers, "twitter:user:list").is_err());
    }

    // ── QA: PRD-0004 additional tests ──

    fn make_token(secret: &str, claims: &serde_json::Value) -> String {
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn bearer_headers(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(
            "authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );
        h
    }

    #[test]
    fn token_without_role_defaults_to_user() {
        let secret = "qa-secret";
        let mut map = PermissionMap::new();
        map.add_role("user", &["twitter:tweet:create"]);

        let auth = RbacAuthenticator::new(secret, map);
        let token = make_token(
            secret,
            &serde_json::json!({
                "sub": "alice",
                "iat": chrono::Utc::now().timestamp(),
                "exp": chrono::Utc::now().timestamp() + 3600,
            }),
        );
        let headers = bearer_headers(&token);
        assert!(auth.check(&headers, "twitter:tweet:create").is_ok());
    }

    #[test]
    fn expired_token_returns_unauthorized() {
        let secret = "qa-secret";
        let mut map = PermissionMap::new();
        map.add_role("admin", &["*:*:*"]);
        let auth = RbacAuthenticator::new(secret, map);

        let token = make_token(
            secret,
            &serde_json::json!({
                "sub": "alice", "role": "admin",
                "iat": chrono::Utc::now().timestamp() - 7200,
                "exp": chrono::Utc::now().timestamp() - 3600,
            }),
        );
        let headers = bearer_headers(&token);
        let err = auth.check(&headers, "any:perm:here").unwrap_err();
        assert!(
            matches!(err, ServiceError::Unauthorized(_)),
            "expected Unauthorized, got {:?}",
            err
        );
    }

    #[test]
    fn wrong_secret_returns_unauthorized() {
        let mut map = PermissionMap::new();
        map.add_role("admin", &["*:*:*"]);
        let auth = RbacAuthenticator::new("verifier-secret", map);

        let token = make_token(
            "issuer-secret",
            &serde_json::json!({
                "sub": "alice", "role": "admin",
                "iat": chrono::Utc::now().timestamp(),
                "exp": chrono::Utc::now().timestamp() + 3600,
            }),
        );
        let headers = bearer_headers(&token);
        let err = auth.check(&headers, "any:perm:here").unwrap_err();
        assert!(matches!(err, ServiceError::Unauthorized(_)));
    }

    #[test]
    fn missing_header_is_unauthorized_not_forbidden() {
        let auth = RbacAuthenticator::new("secret", PermissionMap::new());
        let err = auth.check(&HeaderMap::new(), "x:y:z").unwrap_err();
        assert!(
            matches!(err, ServiceError::Unauthorized(_)),
            "expected Unauthorized, got {:?}",
            err
        );
    }

    #[test]
    fn valid_token_insufficient_role_is_permission_denied() {
        let secret = "qa-secret";
        let mut map = PermissionMap::new();
        map.add_role("user", &["twitter:tweet:read"]);
        let auth = RbacAuthenticator::new(secret, map);

        let token = make_token(
            secret,
            &serde_json::json!({
                "sub": "alice", "role": "user",
                "iat": chrono::Utc::now().timestamp(),
                "exp": chrono::Utc::now().timestamp() + 3600,
            }),
        );
        let headers = bearer_headers(&token);
        let err = auth.check(&headers, "twitter:user:delete").unwrap_err();
        assert!(
            matches!(err, ServiceError::PermissionDenied(_)),
            "expected PermissionDenied, got {:?}",
            err
        );
    }

    #[test]
    fn empty_required_permission_no_match() {
        assert!(!permission_matches("", "a:b:c"));
        assert!(!permission_matches("a:b:c", ""));
    }

    #[test]
    fn wildcard_module_exact_action() {
        assert!(permission_matches(
            "twitter:tweet:create",
            "twitter:*:create"
        ));
        assert!(!permission_matches(
            "twitter:tweet:delete",
            "twitter:*:create"
        ));
    }
}
