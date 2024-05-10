use crate::data_models::role::Role;
use getset::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Getters)]
#[getset(get = "pub")]
pub struct User {
    name: String,
    email: String,
    oid: Uuid,
    sub: String,
    roles: Vec<Role>,
}
