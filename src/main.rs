use crate::controllers::auth_controller::{callback, login, validate};
use crate::shared::{
    oidc_client_factory::init_oidc_client, oidc_state::OidcState, settings::Settings,
};
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use getset::Getters;
use log::{debug, info};
use serde_json::to_string;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

mod controllers;
mod handlers;
mod models;
mod shared;

#[derive(Getters, Clone)]
#[getset(get = "pub")]
struct AppState {
    settings: Arc<Settings>,
    oidc_state_map: Arc<Mutex<HashMap<String, OidcState>>>,
}

impl AppState {
    fn new(settings: Settings) -> Self {
        Self {
            settings: Arc::new(settings),
            oidc_state_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    info!("Started auth-service!");

    let settings = Settings::new().expect("Failed to load settings");
    let app_state = AppState::new(settings);
    debug!("App state loaded!");

    let data = web::Data::new(app_state.clone());

    let app = move || {
        App::new()
            .app_data(data.clone())
            .route("/validate", web::get().to(validate))
            .route("/login", web::get().to(login))
            .route("/callback", web::get().to(callback))
    };

    HttpServer::new(app).bind("127.0.0.1:8088")?.run().await
}
