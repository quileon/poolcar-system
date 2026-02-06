use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};

use crate::{error::AppError, types::Claims};

/// Hash password using Argon2 algorithm.
pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;

    Ok(password_hash.to_string())
}

/// Verify if the password is the same with the stored hash. Returns true if match, false otherwise.
pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(stored_hash) {
        Ok(hash) => hash,
        Err(_) => return false,
    };

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Decode JWT token back into claim.
///
/// Requires the encoded JWT string and the secret key.
pub fn decode_jwt(token: &str, secret: &str) -> Result<TokenData<Claims>, AppError> {
    let result = decode(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| AppError::InvalidToken)?;

    Ok(result)
}

/// Crate JWT token by encoding claims.
///
/// Requires the claims and the secret key.
pub fn encode_jwt(claims: Claims, secret: &str) -> Result<String, AppError> {
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::EncodingError)?;

    Ok(token)
}
