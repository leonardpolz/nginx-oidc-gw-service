use log::info;
use surrealdb::{engine::remote::ws::Client, Error, Surreal};

use crate::data_models::user::User;

pub struct DbContext {
    client: Surreal<Client>,
}

impl DbContext {
    pub fn new(client: Surreal<Client>) -> Self {
        Self { client }
    }

    pub async fn fetch_user_by_id(&self, id: String) -> Result<Option<User>, Error> {
        info!("Fetching user with ID: {}", id);

        let query_result: Result<Option<User>, Error> = self.client.select(("user", id)).await;

        let user = match query_result {
            Ok(result) => result,
            Err(err) => {
                log::error!("Failed to query user roles: {:?}", err);
                return Err(err);
            }
        };

        Ok(user)
    }

    pub async fn patch_user(&self, user: User) -> Result<String, Error> {
        info!("Patching user: {:?}", user);

        let existing_user = self.fetch_user_by_id(user.oid().to_string()).await?;

        if let Some(existing_user) = existing_user {
            info!("User exists, updating user");
            let _: Result<Option<Vec<User>>, Error> = self
                .client
                .update(("user", existing_user.oid()))
                .content(user)
                .await;

            return Ok("User updated successfully".to_string());
        }

        info!("User does not exist, creating user");
        let _: Result<Option<Vec<User>>, Error> =
            self.client.create(("user", user.oid())).content(user).await;

        Ok("Created new User".to_string())
    }
}
