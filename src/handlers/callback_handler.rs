use crate::models::role::Role;
use crate::models::user::User;
use crate::shared::jwt_provider::generate_jwt;
use crate::shared::oidc_state::OidcState;
use crate::shared::settings::Settings;
use actix_web::{HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use base64::prelude::*;
use log::{error, info, warn};
use openidconnect::{
    core::CoreClient, reqwest::async_http_client, AuthorizationCode, TokenResponse,
};
use serde_json::from_str;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub async fn handle(
    request: HttpRequest,
    settings: &Settings,
    oidc_state_map: &Arc<Mutex<HashMap<String, OidcState>>>,
    oidc_client: &CoreClient,
) -> impl Responder {
    let state = extract_query_param(&request, "state");
    let oidc_code = extract_query_param(&request, "code");
    let mut oidc_state = oidc_state_map.lock().unwrap().remove(&state).unwrap();

    let nginx_redirect_uri = oidc_state.redirect_uri().clone();

    let pkce_verifier = match oidc_state.take_pkce_verifier() {
        Some(pkce_verifier) => pkce_verifier,
        None => {
            error!("PKCE Verifier not found");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(oidc_code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(async_http_client)
        .await
        .expect("Failed to exchange code for token");

    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))
        .expect("Failed to get ID token");

    let nonce = match oidc_state.take_nonce() {
        Some(nonce) => nonce,
        None => {
            error!("Nonce not found");
            return HttpResponse::InternalServerError().finish();
        }
    };

    let claims = id_token
        .claims(&oidc_client.id_token_verifier(), &nonce)
        .expect("Failed to verify ID token");

    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        claims
            .email()
            .map(|email| email.as_str())
            .unwrap_or("<not provided>")
    );

    let test_user = User::new(
        Uuid::new_v4(),
        "test_user".to_string(),
        "test@user.de".to_string(),
        vec![Role::new(Uuid::new_v4(), "test_role".to_string())],
    );

    let token = generate_jwt(test_user, settings.jwt());

    let cookie_value = format!("auth_token={}; HttpOnly; Path=/", token);

    info!("Redirecting to {}", nginx_redirect_uri);
    HttpResponse::Found()
        .append_header(("Set-Cookie", cookie_value))
        .append_header(("Location", nginx_redirect_uri))
        .finish()
}

fn extract_query_param(request: &HttpRequest, param_name: &str) -> String {
    request
        .query_string()
        .split('&')
        .find_map(|param| {
            let mut parts = param.split('=');
            if parts.next()? == param_name {
                parts.next().map(|v| v.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "/".to_string())
}
