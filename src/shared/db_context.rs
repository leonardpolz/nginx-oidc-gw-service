use log::info;
use surrealdb::{engine::remote::ws::Client, Response, Surreal};

use crate::data_models::user::User;

pub struct DbContext {
    client: Surreal<Client>,
}

impl DbContext {
    pub fn new(client: Surreal<Client>) -> Self {
        Self { client }
    }

    pub async fn fetch_user_by_id(self, id: String) -> Option<User> {
        info!("Fetching user with ID: {}", id);
        let mut users_result: Response = self
            .client
            .query(format!("SELECT * FROM user:{};", id))
            .await
            .expect("Failed to query user roles");

        let users: Vec<User> = users_result
            .take::<Vec<User>>(0)
            .expect("Failed to get user");

        info!("Fetched user: {:?}", users);

        users.into_iter().next()
    }
}
