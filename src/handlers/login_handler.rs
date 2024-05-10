use crate::shared::oidc_state::OidcState;
use actix_web::{HttpRequest, HttpResponse, Responder};
use log::{debug, info};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient},
    CsrfToken, Nonce, PkceCodeChallenge, Scope,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

pub async fn handle(
    request: HttpRequest,
    oidc_state_map: &Arc<Mutex<HashMap<String, OidcState>>>,
    oidc_client: &CoreClient,
) -> impl Responder {
    let nginx_redirect_uri = request
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

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user
    debug!("Generating authorization URL...");
    let (auth_url, csrf_token, nonce) = oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("email".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let oidc_state = OidcState::new(nginx_redirect_uri.to_string(), pkce_verifier, nonce.clone());

    oidc_state_map
        .lock()
        .unwrap()
        .insert(csrf_token.secret().to_string(), oidc_state);

    info!("Redirecting to {}", auth_url.to_string());
    HttpResponse::Found()
        .append_header(("Location", auth_url.to_string()))
        .finish()
}
