use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct AuthService {
    secret: String,
}

impl AuthService {
    pub fn new() -> Self {
        AuthService {
            secret: std::env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        }
    }

    pub fn create_token(&self, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 86400; // 24h
        let claims = Claims { sub: username.to_string(), exp };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
    }
}
