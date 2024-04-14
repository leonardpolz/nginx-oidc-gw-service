use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
pub struct Role {
    id: Uuid,
    name: String,
}

impl Role {
    pub fn new(id: Uuid, name: String) -> Role {
        Role { id, name }
    }
}
