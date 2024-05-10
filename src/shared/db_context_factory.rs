use crate::shared::{db_context::DbContext, settings::DbSettings};
use log::info;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Surreal};

pub async fn init_db_context(settings: &DbSettings) -> DbContext {
    info!("Initializing database context...");
    let client = Surreal::new::<Ws>(&settings.connection_string().to_string())
        .await
        .expect("Failed to connect to database");

    info!(
        "Signing in to database using username {}...",
        settings.username()
    );

    let _ = client
        .signin(Root {
            username: settings.username(),
            password: settings.password(),
        })
        .await;

    info!(
        "Using namespace {} and database {}...",
        settings.namespace(),
        settings.database()
    );

    let _ = client
        .use_ns(&settings.namespace().to_string())
        .use_db(&settings.database().to_string())
        .await;

    DbContext::new(client)
}
