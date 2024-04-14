use crate::shared::jwt_provider::validate_jwt;
use crate::shared::settings::JwtSettings;
use crate::shared::token_cache::TOKEN_CACHE;
use actix_web::{HttpRequest, HttpResponse, Responder};
use log::info;

pub async fn handle(request: HttpRequest, jwt_settings: &JwtSettings) -> impl Responder {
    let token = request
        .cookie("auth_token")
        .map(|cookie| cookie.value().to_string());

    match token {
        Some(t) if TOKEN_CACHE.lock().unwrap().contains(&t) => {
            let message = "Token is cached.";
            info!("{}", message);
            HttpResponse::Ok().body(message)
        }
        Some(t) => {
            if validate_jwt(&t, &jwt_settings).is_ok() {
                let message = "Token is valid.";
                info!("{}", message);
                TOKEN_CACHE.lock().unwrap().insert(t);
                HttpResponse::Ok().body(message)
            } else {
                let message = "Token is invalid.";
                info!("{}", message);
                HttpResponse::Unauthorized().body(message)
            }
        }
        None => {
            let message = "No token found in request.";
            info!("{}", message);
            HttpResponse::Unauthorized().body(message)
        }
    }
}
