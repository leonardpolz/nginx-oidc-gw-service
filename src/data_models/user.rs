use crate::data_models::role::Role;
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Getters)]
#[getset(get = "pub")]
pub struct User {
    oid: String,
    name: String,
    email: String,
    roles: Vec<Role>,
}

impl User {
    pub fn new(oid: String, email: String, roles: Vec<Role>) -> Self {
        Self {
            oid,
            name: "".to_string(),
            email,
            roles,
        }
    }
}
