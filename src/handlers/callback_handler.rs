use crate::shared::db_context::DbContext;
use crate::shared::jwt_provider::generate_jwt;
use crate::shared::oidc_state::OidcState;
use crate::shared::settings::Settings;
use actix_web::cookie::Cookie;
use actix_web::{HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use log::info;
use openidconnect::{
    core::CoreClient, reqwest::async_http_client, AuthorizationCode, TokenResponse,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use openidconnect::AdditionalClaims;
use serde::Deserialize;

pub async fn handle(
    request: HttpRequest,
    settings: &Settings,
    oidc_state_map: &Arc<Mutex<HashMap<String, OidcState>>>,
    oidc_client: &CoreClient,
    db_context: DbContext,
) -> impl Responder {
    let state = extract_query_param(&request, "state");
    let oidc_code = extract_query_param(&request, "code");

    if !oidc_state_map
        .try_lock()
        .expect("Failed to lock OIDC state map")
        .contains_key(&state)
    {
        return HttpResponse::Unauthorized().finish();
    }

    let mut oidc_state = oidc_state_map
        .try_lock()
        .expect("Failed to lock OIDC state map")
        .remove(&state)
        .expect("State not found");

    let nginx_redirect_uri = oidc_state.redirect_uri().clone();

    let pkce_verifier = oidc_state
        .take_pkce_verifier()
        .expect("PKCE verfier not found");

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

    let nonce = oidc_state.take_nonce().expect("Nonce not found");

    let claims = id_token
        .claims(&oidc_client.id_token_verifier(), &nonce)
        .expect("Failed to verify ID token");

    let email = claims
        .email()
        .map(|email| email)
        .expect("Failed to get e-mail address");

    println!(
        "User {} with e-mail address {} has authenticated successfully",
        claims.subject().as_str(),
        email.as_str(),
    );

    let user_result = db_context.fetch_user_by_email(email.to_string()).await;

    info!("User result: {:?}", user_result);

    match user_result {
        None => {
            info!("User not found in database");
            HttpResponse::Unauthorized().finish()
        }

        Some(user) => {
            let token = generate_jwt(user, settings.jwt());

            let cookie = Cookie::build("auth_token", token)
                .path("/")
                .secure(false) // Temporarily set to false for testing
                .http_only(false) // Temporarily set to false for testing
                .finish();

            info!("Redirecting to {}", nginx_redirect_uri);
            HttpResponse::Found()
                .cookie(cookie)
                .append_header(("Location", nginx_redirect_uri))
                .finish()
        }
    }
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct MyAdditionalClaims {
    pub oid: String,
}

impl AdditionalClaims for MyAdditionalClaims {}
