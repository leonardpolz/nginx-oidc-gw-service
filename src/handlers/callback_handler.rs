use crate::models::role::Role;
use crate::models::user::User;
use crate::shared::jwt_provider::generate_jwt;
use crate::shared::oidc_state::OidcState;
use crate::shared::settings::Settings;
use actix_web::{HttpRequest, HttpResponse, Responder};
use anyhow::anyhow;
use base64::prelude::*;
use log::{info, warn};
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
    let state = request
        .query_string()
        .split('&')
        .find_map(|param| {
            let mut parts = param.split('=');
            if parts.next()? == "state" {
                parts.next()
            } else {
                None
            }
        })
        .unwrap_or("/")
        .to_string();

    let oidc_code = request
        .query_string()
        .split('&')
        .find_map(|param| {
            let mut parts = param.split('=');
            if parts.next()? == "code" {
                parts.next()
            } else {
                None
            }
        })
        .unwrap_or("/");

    let oidc_state = oidc_state_map.lock().unwrap().remove(&state).unwrap();

    let nginx_redirect_uri = oidc_state.redirect_uri().clone();

    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(oidc_code.to_string()))
        .set_pkce_verifier(oidc_state.take_pkce_verifier())
        .request_async(async_http_client)
        .await
        .expect("Failed to exchange code for token");

    let id_token = token_response
        .id_token()
        .ok_or_else(|| anyhow!("Server did not return an ID token"))
        .expect("Failed to get ID token");

    info!("ID token: {:?}", id_token);

    // let claims = id_token.claims(&oidc_client.id_token_verifier(), move || oicd_state.toke_nonce().clone())

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
