use crate::data_models::role::Role;
use crate::data_models::user::User;
use crate::shared::settings::JwtSettings;
use jsonwebtoken::{
    decode, encode, errors::ErrorKind, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    oid: Uuid,
    exp: u64,
    iss: String,
    aud: String,
    email: String,
    name: String,
    roles: Vec<Role>,
}

pub fn generate_jwt(user: User, jwt_settings: &JwtSettings) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        exp: now + 3600,
        iss: "test".to_string(),
        aud: "test".to_string(),
        oid: Uuid::from_str(user.oid()).expect("Failed to parse UUID"),
        email: user.email().to_string(),
        name: user.name().to_string(),
        roles: user.roles().to_vec(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_settings.secret().as_ref()),
    )
    .expect("Error encoding JWT")
}

pub fn validate_jwt(
    token: &str,
    jwt_settings: &JwtSettings,
) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
    let mut validation = Validation::default();
    validation.iss = Some(HashSet::from_iter(vec!["test".to_string()]));
    validation.aud = Some(HashSet::from_iter(vec!["test".to_string()]));

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_settings.secret().as_ref()),
        &validation,
    ) {
        Ok(token_data) => Ok(token_data),
        Err(err) => {
            match *err.kind() {
                ErrorKind::InvalidToken => debug!("Token is invalid"),
                ErrorKind::ExpiredSignature => debug!("Token has expired"),
                ErrorKind::InvalidIssuer => debug!("Issuer is invalid"),
                ErrorKind::InvalidAudience => debug!("Audience is invalid"),
                _ => warn!("Some other token error occurred: {:?}", err),
            }
            Err(err)
        }
    }
}
