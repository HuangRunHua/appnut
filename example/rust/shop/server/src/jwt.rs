//! JWT service for ShopFlux — issue and verify tokens.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub name: String,
    pub role: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: jsonwebtoken::EncodingKey,
    decoding_key: jsonwebtoken::DecodingKey,
    validation: jsonwebtoken::Validation,
    expire_secs: i64,
}

pub const SHOP_TEST_SECRET: &str = "shop-test-jwt-secret";

impl JwtService {
    pub fn new(secret: &str, expire_secs: i64) -> Self {
        Self {
            encoding_key: jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
            validation: jsonwebtoken::Validation::default(),
            expire_secs,
        }
    }

    pub fn shop_test() -> Self {
        Self::new(SHOP_TEST_SECRET, 86400)
    }

    pub fn issue(&self, user_id: &str, display_name: &str, role: &str) -> Result<String, String> {
        let now = chrono::Utc::now().timestamp();
        let claims = Claims {
            sub: user_id.to_string(),
            name: display_name.to_string(),
            role: role.to_string(),
            iat: now,
            exp: now + self.expire_secs,
        };
        jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &self.encoding_key,
        )
        .map_err(|e| format!("jwt encode: {}", e))
    }

    pub fn verify(&self, token: &str) -> Result<Claims, String> {
        jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| format!("jwt verify: {}", e))
    }
}
