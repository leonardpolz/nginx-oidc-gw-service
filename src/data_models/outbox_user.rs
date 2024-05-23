use crate::data_models::role::Role;
use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Getters)]
#[getset(get = "pub")]
pub struct OutboxUser {
    id: Uuid,
    email: String,
    roles: Vec<Role>,
    action: OutboxAction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum OutboxAction {
    CREATE,
    DELETE,
}
