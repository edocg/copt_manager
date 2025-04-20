use std::env;
use dotenvy::dotenv;

use serde::{Serialize, Deserialize};
use jsonwebtoken::{encode, EncodingKey, Header};
//use std::time::{SystemTime, UNIX_EPOCH};
use chrono::Utc;
use chrono::Duration;

pub fn get_jwt_secret() -> String {
    dotenv().ok(); // Carga el archivo .env
    env::var("JWT_SECRET").expect("JWT_SECRET no definida en .env")
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: u32,          // ID del residente
    pub name: String,      // Nombre del residente
    pub role: String,
    pub exp: usize,        // Fecha de expiración (timestamp)
}

/// Genera un token JWT para un residente
pub fn generate_token(id: u32, name: String, role: String) -> String {
    

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: id,
        name,
        role,
        exp: expiration as usize,
    };

    let secret = get_jwt_secret();

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    ).unwrap()
}

use jsonwebtoken::{decode, DecodingKey, Validation};

/// Verifica el token y devuelve los claims (datos del residente)
pub fn verify_token(token: &str) -> Option<Claims> {
    let secret = get_jwt_secret();

    let validation = Validation::default();

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(token_data) => Some(token_data.claims),
        Err(_) => None,
    }
}

use actix_web::{FromRequest, HttpRequest, dev::Payload, Error};
use futures_util::future::{ready, Ready};

pub struct AuthenticatedUser {
    pub id: u32,
    pub name: String,
    pub role: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if let Some(header_value) = req.headers().get("Authorization") {
            if let Ok(auth_str) = header_value.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    if let Some(claims) = verify_token(token) {
                        return ready(Ok(AuthenticatedUser {
                            id: claims.sub,
                            name: claims.name,
                            role: claims.role,
                        }));
                    }
                }
            }
        }

        ready(Err(actix_web::error::ErrorUnauthorized("Token inválido")))
    }
}
