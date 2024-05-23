use crate::shared::oidc_state::OidcState;
use actix_web::{HttpRequest, HttpResponse, Responder};
use log::info;
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient},
    AdditionalClaims, CsrfToken, Nonce, PkceCodeChallenge, Scope,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use urldecode::decode;

pub async fn handle(
    request: HttpRequest,
    oidc_state_map: &Arc<Mutex<HashMap<String, OidcState>>>,
    oidc_client: &CoreClient,
) -> impl Responder {
    let nginx_redirect_uri_raw = request
        .query_string()
        .split('&')
        .find_map(|param| {
            let mut parts = param.split('=');
            if parts.next()? == "rd" {
                parts.next()
            } else {
                None
            }
        })
        .unwrap_or("/");

    let nginx_redirect_uri = decode(nginx_redirect_uri_raw.to_string());

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    info!("Generating authorization URL...");
    let (auth_url, csrf_token, nonce) = oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let oidc_state = OidcState::new(nginx_redirect_uri, pkce_verifier, nonce.clone());

    oidc_state_map
        .try_lock()
        .expect("Failed to lock OIDC state map")
        .insert(csrf_token.secret().to_string(), oidc_state);

    info!("Redirecting to {}", auth_url.to_string());
    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}
