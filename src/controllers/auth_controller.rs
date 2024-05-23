use crate::handlers::{callback_handler, login_handler, validation_handler};
use crate::shared::db_context_factory::init_db_context;
use crate::shared::oidc_client_factory::init_oidc_client;
use crate::AppState;
use actix_web::{web, HttpRequest, Responder};
use log::info;

pub async fn validate(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let remote_addr = req
        .peer_addr()
        .map_or_else(|| "unknown".to_string(), |addr| addr.to_string());
    info!("Processing token validation request from {}", remote_addr);

    validation_handler::handle(req, app_state.settings().jwt()).await
}

pub async fn login(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let remote_addr = req
        .peer_addr()
        .map_or_else(|| "unknown".to_string(), |addr| addr.to_string());
    info!("Processing login request from {}", remote_addr);

    let oidc_client = init_oidc_client(app_state.settings().entra()).await;
    login_handler::handle(req, app_state.oidc_state_map(), &oidc_client).await
}

pub async fn callback(app_state: web::Data<AppState>, req: HttpRequest) -> impl Responder {
    let remote_addr = req
        .peer_addr()
        .map_or_else(|| "unknown".to_string(), |addr| addr.to_string());
    info!("Processing callback request from {}", remote_addr);

    let oidc_client = init_oidc_client(app_state.settings().entra()).await;
    let db_context = init_db_context(app_state.settings().db()).await;

    callback_handler::handle(
        req,
        app_state.settings(),
        app_state.oidc_state_map(),
        &oidc_client,
        db_context,
    )
    .await
}
