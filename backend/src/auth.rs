use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error as JwtError,
};
use rocket::State;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject: username
    pub exp: usize,   // Expiry timestamp
    pub role: String, // User role (e.g. Admin, Employee)
}

pub struct AuthenticatedUser {
    pub username: String,
    pub role: String,
    pub token: String, // <-- Added raw token string
}

/// Managed state containing the JWT secret key
pub struct JwtSecret(pub Vec<u8>);

/// Generate a JWT token valid for 24 hours using the managed secret key
pub fn create_token(username: &str, role: &str, secret: &[u8]) -> Result<String, JwtError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(1))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration,
        role: role.to_owned(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

/// Decode and validate a JWT token using the managed secret key
pub fn decode_token(token: &str, secret: &[u8]) -> Result<Claims, JwtError> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Fetch the managed JWT secret from Rocket's state
        let secret = match request.guard::<&State<JwtSecret>>().await {
            Outcome::Success(s) => &s.0,
            _ => {
                // If the state is not registered in main.rs, fail request
                return Outcome::Error((Status::InternalServerError, ()));
            }
        };

        // 1. Try to extract token from the "Authorization" header (Bearer token)
        if let Some(auth_header) = request.headers().get_one("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                if let Ok(claims) = decode_token(token, secret) {
                    return Outcome::Success(AuthenticatedUser {
                        username: claims.sub,
                        role: claims.role,
                        token: token.to_owned(), // <-- Store it!
                    });
                }
            }
        }

        // 2. Fallback: Try to extract token from the "session_token" cookie
        if let Some(cookie) = request.cookies().get("session_token") {
            let token = cookie.value();
            if let Ok(claims) = decode_token(token, secret) {
                return Outcome::Success(AuthenticatedUser {
                    username: claims.sub,
                    role: claims.role,
                    token: token.to_owned(), // <-- Store it!
                });
            }
        }

        // If neither is found or valid, return Unauthorized
        Outcome::Error((Status::Unauthorized, ()))
    }
}
