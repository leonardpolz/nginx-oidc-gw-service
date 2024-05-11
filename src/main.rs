use crate::controllers::auth_controller::{callback, login, validate};
use crate::shared::db_context_factory::init_db_context;
use crate::shared::{oidc_state::OidcState, settings::Settings};
use actix_rt::spawn;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use dotenv::dotenv;
use getset::Getters;
use kafka::client::{FetchOffset, GroupOffsetStorage};
use kafka::consumer::Consumer;
use log::{debug, info};
use shared::settings::{DbSettings, KafkaSettings};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

mod controllers;
mod data_models;
mod handlers;
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
    let app_state = AppState::new(settings.clone());
    debug!("App state loaded!");

    info!("Starting db sync task...");
    spawn(db_sync_task(
        settings.kafka().clone(),
        settings.db().clone(),
    ));

    let data = web::Data::new(app_state.clone());

    let app = move || {
        App::new()
            .app_data(data.clone())
            .wrap(Logger::default())
            .route("/validate", web::get().to(validate))
            .route("/login", web::get().to(login))
            .route("/callback", web::get().to(callback))
    };

    HttpServer::new(app)
        .bind(settings.app().bind())?
        .run()
        .await
}

async fn db_sync_task(kafka_settings: KafkaSettings, db_settings: DbSettings) {
    let mut consumer = Consumer::from_hosts(kafka_settings.brokers().to_owned())
        .with_topic_partitions(kafka_settings.topic_name().to_owned(), &[0, 1])
        .with_fallback_offset(FetchOffset::Earliest)
        .with_group(kafka_settings.group_name().to_owned())
        .with_offset_storage(Some(GroupOffsetStorage::Kafka))
        .create()
        .expect("Failed to create consumer");

    loop {
        for ms in consumer.poll().expect("Failed to poll").iter() {
            for m in ms.messages() {
                info!("Received message: {:?}", m);
                let db_context = init_db_context(&db_settings).await;
                let user = serde_json::from_slice(m.value).expect("Failed to deserialize user");
                db_context
                    .patch_user(user)
                    .await
                    .expect("Failed to patch user");
            }
            let _ = consumer.consume_messageset(ms);
        }
        consumer
            .commit_consumed()
            .expect("Failed to commit consumed messages");
    }
}
